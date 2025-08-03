use std::{future::Future, sync::Arc};

use crate::persistence::{PersistenceObject, Store};
use cloudevents::{event::ExtensionValue, AttributesReader, Data, Event};
use errors::CQRLResult;
use futures::StreamExt;
use parser::API;
use serde_json::Value;
use tracing::debug;

pub trait EventEmitter<S>: Send + Sync
where
    S: Store,
{
    fn run(self: &mut Self) -> impl Future<Output = CQRLResult<()>> + Send;
    fn emit(self: &mut Self, event: PersistenceObject) -> CQRLResult<()>;
    fn listen(self: &mut Self, event: Value) -> impl StreamExt<Item = Event> + Send;
    fn listen_permission(self: &mut Self, event: Value) -> impl StreamExt<Item = Event> + Send;
}

fn validator(api: Arc<API>) -> impl Fn(Arc<Event>) -> CQRLResult<Event> {
    move |event| validate_event(event, api.clone())
}

fn permission_validator(api: Arc<API>) -> impl Fn(Arc<Event>) -> CQRLResult<Event> {
    move |event| validate_permission(event, api.clone())
}

fn validate_event(event: Arc<Event>, api: Arc<API>) -> CQRLResult<Event> {
    println!("Validating event: {:?}", event.id());
    let event_type = event.ty();
    match api.queries.iter().find(|q| q.name == event_type) {
        Some(query) => {
            let model = api
                .models
                .iter()
                .find(|m| m.name == query.modelled_by)
                .unwrap();
            let event_data = match event.data() {
                Some(data) => match data {
                    Data::Json(json) => json.clone(),
                    Data::String(string) => serde_json::from_str(&string).unwrap(),
                    Data::Binary(binary) => serde_json::from_slice(&binary).unwrap(),
                },
                None => return Err(errors::CQRLError::NoEventData),
            };

            for property in model.properties.iter() {
                let value = event_data.get(property.name.clone());
                if property.required && value.is_none() {
                    return Err(errors::CQRLError::RequiredFieldNotSet {
                        name: property.name.clone(),
                    });
                }
                match value {
                    Some(value) => match (value.is_array(), property.list) {
                        (true, true) => {
                            let value_as_vec = value.as_array().unwrap();
                            for val in value_as_vec.iter() {
                                let check = validate_datatype(
                                    property.name.clone(),
                                    val,
                                    property.datatype.clone(),
                                );

                                if check.is_err() {
                                    return Err(check.err().unwrap());
                                }
                            }
                        }
                        (true, false) => {
                            return Err(errors::CQRLError::IncorrectTypeForField {
                                name: property.name.clone(),
                                ty: format!("Vec<{}>", property.datatype),
                            })
                        }
                        (false, true) => {
                            return Err(errors::CQRLError::IncorrectTypeForField {
                                name: property.name.clone(),
                                ty: format!("{}", property.datatype),
                            })
                        }
                        (false, false) => {
                            let check = validate_datatype(
                                property.name.clone(),
                                value,
                                property.datatype.clone(),
                            );

                            if check.is_err() {
                                return Err(check.err().unwrap());
                            }
                        }
                    },
                    None => {}
                }
            }

            Ok(event.as_ref().clone())
        }
        None => Err(errors::CQRLError::InvalidEventType),
    }
}

fn validate_permission(event: Arc<Event>, api: Arc<API>) -> CQRLResult<Event> {
    println!("Validating event: {:?}", event.id());
    let event_type = event.ty();
    match api.queries.iter().find(|q| q.name == event_type) {
        Some(_) => {
            let event_data = match event.data() {
                Some(data) => match data {
                    Data::Json(json) => json.clone(),
                    Data::String(string) => serde_json::from_str(&string).unwrap(),
                    Data::Binary(binary) => serde_json::from_slice(&binary).unwrap(),
                },
                None => return Err(errors::CQRLError::NoEventData),
            };

            match event.subject() {
                None => return Err(errors::CQRLError::NoEventData),
                Some(_) => (),
            }

            debug!("Event: {:?}", event);

            match event.extension("authtype") {
                None => {
                    debug!("No authtype extension found");
                    return Err(errors::CQRLError::InvalidEventType);
                }
                Some(auth) => match auth {
                    ExtensionValue::String(auth) => {
                        if auth == "unauthenticated" {
                            debug!("Unauthenticated auth type");
                            return Err(errors::CQRLError::InvalidEventType);
                        }
                    }
                    _ => {
                        debug!("Invalid auth type");
                        return Err(errors::CQRLError::InvalidEventType);
                    }
                },
            }

            match event.extension("authid") {
                None => {
                    debug!("No authid extension found");
                    return Err(errors::CQRLError::InvalidEventType);
                }
                Some(_) => (),
            }

            match event_data.get("type") {
                None => {
                    return Err(errors::CQRLError::RequiredFieldNotSet {
                        name: "type".to_string(),
                    });
                }
                Some(ty) => match ty {
                    Value::String(ty) => match ty.as_str() {
                        "permit" => {}
                        "deny" => {}
                        _ => {
                            debug!("Invalid type");
                            return Err(errors::CQRLError::InvalidEventType);
                        }
                    },
                    _ => {
                        return Err(errors::CQRLError::InvalidEventType);
                    }
                },
            }

            Ok(event.as_ref().clone())
        }
        None => Err(errors::CQRLError::InvalidEventType),
    }
}

fn validate_datatype(name: String, value: &Value, datatype: parser::DataTypes) -> CQRLResult<()> {
    match &datatype {
        parser::DataTypes::ID => {
            if !value.is_string() {
                return Err(errors::CQRLError::IncorrectTypeForField {
                    name: name.clone(),
                    ty: datatype.to_string(),
                });
            }
            if ulid::Ulid::from_string(value.as_str().unwrap()).is_err() {
                return Err(errors::CQRLError::IncorrectTypeForField {
                    name: name.clone(),
                    ty: datatype.to_string(),
                });
            }
            Ok(())
        }
        parser::DataTypes::String => {
            if !value.is_string() {
                return Err(errors::CQRLError::IncorrectTypeForField {
                    name: name.clone(),
                    ty: datatype.to_string(),
                });
            }
            Ok(())
        }
        parser::DataTypes::Boolean => {
            if !value.is_boolean() {
                return Err(errors::CQRLError::IncorrectTypeForField {
                    name: name.clone(),
                    ty: datatype.to_string(),
                });
            }
            Ok(())
        }
        parser::DataTypes::Datetime => {
            if !value.is_string() {
                return Err(errors::CQRLError::IncorrectTypeForField {
                    name: name.clone(),
                    ty: datatype.to_string(),
                });
            }
            if chrono::DateTime::parse_from_rfc3339(value.as_str().unwrap()).is_err() {
                return Err(errors::CQRLError::IncorrectTypeForField {
                    name: name.clone(),
                    ty: datatype.to_string(),
                });
            }
            Ok(())
        }
        parser::DataTypes::Number => {
            if !value.is_number() {
                return Err(errors::CQRLError::IncorrectTypeForField {
                    name: name.clone(),
                    ty: datatype.to_string(),
                });
            }
            Ok(())
        }
        parser::DataTypes::Model(_) => {
            if !value.is_string() {
                return Err(errors::CQRLError::IncorrectTypeForField {
                    name: name.clone(),
                    ty: datatype.to_string(),
                });
            }
            if ulid::Ulid::from_string(value.as_str().unwrap()).is_err() {
                return Err(errors::CQRLError::IncorrectTypeForField {
                    name: name.clone(),
                    ty: datatype.to_string(),
                });
            }
            Ok(())
        }
        parser::DataTypes::Pattern(pattern) => {
            if !value.is_string() {
                return Err(errors::CQRLError::IncorrectTypeForField {
                    name: name.clone(),
                    ty: datatype.to_string(),
                });
            }
            let regex = regex::Regex::new(pattern).map_err(|_| errors::CQRLError::Generic)?;
            if !regex.is_match(value.as_str().unwrap()) {
                return Err(errors::CQRLError::IncorrectTypeForField {
                    name: name.clone(),
                    ty: datatype.to_string(),
                });
            }
            Ok(())
        }
    }
}

pub mod nats;
