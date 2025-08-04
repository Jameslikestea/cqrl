use std::sync::Arc;
use std::time::Duration;

use crate::persistence::{Permission, PermissionStore};

use super::{PersistenceObject, Store};
use cloudevents::{AttributesReader, Data, Event};
use errors::CQRLResult;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use mongodb::bson::Document;
use mongodb::{
    bson::{self, doc, Bson},
    Client, IndexModel,
};
use parser::API;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::task;

#[derive(Clone)]
pub struct MongoStore {
    client: Client,
    api: Arc<API>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MongoObject {
    #[serde(rename = "_id")]
    id: String,
    metadata: Value,
    data: Value,
}

impl Into<PersistenceObject> for MongoObject {
    fn into(self) -> PersistenceObject {
        PersistenceObject {
            id: self.id,
            metadata: self.metadata,
            data: self.data,
        }
    }
}

impl From<PersistenceObject> for MongoObject {
    fn from(object: PersistenceObject) -> Self {
        Self {
            id: object.id,
            metadata: object.metadata,
            data: object.data,
        }
    }
}

impl MongoStore {
    pub fn new(client: Client, api: Arc<API>) -> Self {
        Self { client, api }
    }

    pub async fn init(&mut self) -> CQRLResult<()> {
        self.client
            .database("cqrl")
            .collection::<MongoObject>("operations")
            .create_index(
                IndexModel::builder()
                    .keys(doc! {"metadata.type": 1})
                    .build(),
            )
            .await
            .unwrap();
        Ok(())
    }
}

impl MongoStore {
    fn event_to_bson(self: &Self, evt: Event) -> Bson {
        println!("Converting event to bson: {:?}", evt.id());
        let data = evt.data().unwrap();
        let value = match data {
            Data::Json(json) => json.clone(),
            Data::String(string) => serde_json::from_str(&string).unwrap(),
            Data::Binary(binary) => serde_json::from_slice(&binary).unwrap(),
        };

        let mut document = bson::Document::new();
        document.insert("metadata.type", evt.ty());
        document.insert("metadata.time", evt.time().unwrap().to_rfc3339());

        let api = self.api.clone();
        let command = api
            .queries
            .iter()
            .find(|command| command.name == evt.ty())
            .unwrap();
        let model = api
            .models
            .iter()
            .find(|model| model.name == command.modelled_by)
            .unwrap();

        for property in model.properties.iter() {
            let value = value.get(property.name.clone());
            if value.is_some() {
                document.insert(
                    format!("data.{}", property.name.clone()),
                    bson::to_bson(&value.unwrap()).unwrap(),
                );
            }
        }

        Bson::Document(document)
    }

    async fn store_object_internal(&mut self, _k: String, evt: Event, tries: u8) -> CQRLResult<()> {
        let ty = evt.ty().split(".").last().unwrap().to_string();

        let (_id, query) = match evt.subject() {
            Some(subject) => (
                subject.to_string(),
                doc! {
                    "metadata.type": ty.clone(),
                    // Check that we've not already processed this event for this subject
                    "metadata.lineage": {
                        "$nin": vec![evt.id()],
                    },
                    "_id": subject.to_string(),
                },
            ),
            None => {
                println!("No subject found for event: {:?}", evt.id());
                return Err(errors::CQRLError::Generic);
            }
        };

        let update = doc! {
            "$setOnInsert": {
                "_id": _id,
            },
            "$addToSet": {
                "metadata.lineage": evt.id(),
            },
            "$set": self.event_to_bson(evt.clone()),
        };

        match self
            .client
            .database("cqrl")
            .collection::<MongoObject>("objects")
            .update_one(query, update)
            .upsert(true)
            .await
        {
            Ok(result) => {
                println!(
                    "Applied event: {:?}. Modified: {:?}",
                    evt.id(),
                    result.modified_count
                );
                Ok(())
            }
            Err(err) => {
                println!("Error storing object: {:?}", err);
                if tries < 3 {
                    tokio::time::sleep(Duration::from_millis(u64::from(tries) * 100)).await;
                    Box::pin(self.store_object_internal(_k, evt, tries + 1)).await
                } else {
                    Err(errors::CQRLError::StoreError {
                        error: format!("Error storing object: {:?}", err),
                    })
                }
            }
        }
    }

    async fn grant_internal(
        &mut self,
        id: String,
        user: String,
        permission: Permission,
        ty: String,
        tries: u8,
    ) -> CQRLResult<()> {
        let query = doc! {
            "_id": id.clone(),
            "metadata.type": ty.clone(),
        };

        let mut update = Document::new();
        update.insert(
            "$setOnInsert",
            doc! {
                "_id": id.clone(),
            },
        );

        update.insert(
            "$set",
            doc! {
                "metadata.type": ty.clone(),
            },
        );

        match permission {
            Permission::Read => {
                update.insert(
                    "$addToSet",
                    doc! {
                        "metadata.authcontext.read": Bson::String(user.clone()),
                    },
                );
            }
            Permission::Write => {
                update.insert(
                    "$addToSet",
                    doc! {
                        "metadata.authcontext.read": Bson::String(user.clone()),
                        "metadata.authcontext.write": Bson::String(user.clone()),
                    },
                );
            }
        };

        match self
            .client
            .database("cqrl")
            .collection::<MongoObject>("objects")
            .update_one(query, update)
            .upsert(true)
            .await
        {
            Ok(result) => {
                println!(
                    "Applied grant: {:?}. Modified: {:?}",
                    id, result.modified_count
                );
                Ok(())
            }
            Err(err) => {
                if tries < 3 {
                    tokio::time::sleep(Duration::from_millis(u64::from(tries) * 100)).await;
                    Box::pin(self.grant_internal(id, user, permission, ty, tries + 1)).await
                } else {
                    Err(errors::CQRLError::StoreError {
                        error: format!("Error granting permission: {:?}", err),
                    })
                }
            }
        }
    }

    async fn revoke_internal(
        &mut self,
        id: String,
        user: String,
        permission: Permission,
        ty: String,
        tries: u8,
    ) -> CQRLResult<()> {
        let query = doc! {
            "_id": id.clone(),
            "metadata.type": ty.clone(),
        };

        let mut update = Document::new();

        update.insert(
            "$set",
            doc! {
                "metadata.type": ty.clone(),
            },
        );

        match permission {
            Permission::Read => {
                update.insert(
                    "$pull",
                    doc! {
                        "metadata.authcontext.read": Bson::String(user.clone()),
                        "metadata.authcontext.write": Bson::String(user.clone()),
                    },
                );
            }
            Permission::Write => {
                update.insert(
                    "$pull",
                    doc! {
                        "metadata.authcontext.write": Bson::String(user.clone()),
                    },
                );
            }
        };

        match self
            .client
            .database("cqrl")
            .collection::<MongoObject>("objects")
            .update_one(query, update)
            .await
        {
            Ok(result) => {
                println!(
                    "Applied revoke: {:?}. Modified: {:?}",
                    id, result.modified_count
                );
                Ok(())
            }
            Err(err) => {
                if tries < 3 {
                    tokio::time::sleep(Duration::from_millis(u64::from(tries) * 100)).await;
                    Box::pin(self.revoke_internal(id, user, permission, ty, tries + 1)).await
                } else {
                    Err(errors::CQRLError::StoreError {
                        error: format!("Error revoking permission: {:?}", err),
                    })
                }
            }
        }
    }
}

impl Store for MongoStore {
    async fn store_operation(
        &mut self,
        _key: String,
        _value: Value,
        _operation_type: String,
    ) -> CQRLResult<PersistenceObject> {
        let mongo_object = MongoObject {
            id: ulid::Ulid::new().to_string(),
            metadata: json!({
                "type": _operation_type,
            }),
            data: _value,
        };
        self.client
            .database("cqrl")
            .collection("operations")
            .insert_one(mongo_object.clone())
            .await
            .unwrap();
        Ok(mongo_object.into())
    }

    async fn get_object(
        &self,
        id: Option<String>,
        object_type: String,
    ) -> CQRLResult<serde_json::Value> {
        let query = match id {
            Some(id) => doc! {
                "metadata.type": object_type,
                "_id": id,
            },
            None => doc! {
                "metadata.type": object_type,
            },
        };

        let mut object = self
            .client
            .database("cqrl")
            .collection::<MongoObject>("objects")
            .find(query)
            .limit(100)
            .skip(0)
            .await
            .unwrap();
        let mut objects = Vec::new();
        while let Some(obj) = object.next().await {
            objects.push(obj.unwrap().data);
        }

        Ok(Value::Array(objects))
    }

    async fn store_object(&mut self, _k: String, evt: Event) -> errors::CQRLResult<()> {
        self.store_object_internal(_k, evt, 0).await
    }

    fn watch_operation(
        &mut self,
    ) -> impl futures::StreamExt<Item = super::PersistenceObject> + Send {
        let (mut _sender, receiver) = mpsc::unbounded();

        let client = self.client.clone();
        task::spawn(async move {
            let mut stream = client
                .database("cqrl")
                .collection::<MongoObject>("operations")
                .watch()
                .await
                .unwrap();
            while let Some(response) = stream.next().await {
                if let Ok(object) = response {
                    match _sender.send(object.full_document.unwrap().into()).await {
                        Ok(_) => (),
                        Err(err) => {
                            println!("Error sending object: {:?}", err);
                        }
                    }
                }
            }
        });

        receiver
    }
}

impl PermissionStore for MongoStore {
    async fn grant(
        &mut self,
        id: String,
        user: String,
        ty: String,
        permission: super::Permission,
    ) -> CQRLResult<()> {
        self.grant_internal(id, user, permission, ty, 0).await
    }

    async fn revoke(
        &mut self,
        id: String,
        user: String,
        ty: String,
        permission: super::Permission,
    ) -> CQRLResult<()> {
        self.revoke_internal(id, user, permission, ty, 0).await
    }
}
