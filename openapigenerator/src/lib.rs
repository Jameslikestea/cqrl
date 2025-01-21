use std::vec;

use indexmap::IndexMap;
use openapiv3::{
    ArrayType, BooleanType, Components, Header, HeaderStyle, Info, License, NumberType, ObjectType,
    OpenAPI, Operation, PathItem, Paths, ReferenceOr, Response, Responses, Schema, SchemaData,
    SchemaKind, StringFormat, StringType, Type, VariantOrUnknownOrEmpty,
};
use parser::{Command, DataTypes, Model, Query, API};

pub fn generate_openapi_spec(api: API) -> OpenAPI {
    let mut schemas: IndexMap<String, ReferenceOr<Schema>> = IndexMap::new();
    let mut responses: IndexMap<String, ReferenceOr<Response>> = IndexMap::new();

    {
        generate_models(&mut schemas, api.models.clone());
        generate_responses(&mut responses, api.queries.clone());
    }

    OpenAPI {
        openapi: String::from("3.0.0"),
        info: Info {
            title: String::from("CQRL API"),
            license: Some(License {
                name: String::from("MIT"),
                ..Default::default()
            }),
            version: String::from("1.0.0"),
            ..Default::default()
        },
        paths: Paths {
            paths: generate_paths(api.commands.clone(), api.queries.clone()),
            ..Default::default()
        },
        components: Some(Components {
            schemas: schemas,
            responses: responses,
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn generate_paths(
    commands: Vec<Command>,
    queries: Vec<Query>,
) -> IndexMap<String, ReferenceOr<PathItem>> {
    let mut paths = IndexMap::new();

    for command in commands.iter() {
        paths.insert(
            format!("/command.{}", command.name),
            ReferenceOr::Item(PathItem {
                post: Some(Operation {
                    responses: Responses {
                        responses: {
                            let mut imap = IndexMap::new();

                            imap.insert(
                                openapiv3::StatusCode::Code(201),
                                ReferenceOr::ref_("#/components/responses/command.success"),
                            );
                            imap.insert(
                                openapiv3::StatusCode::Code(401),
                                ReferenceOr::ref_("#/components/responses/command.unauthorized"),
                            );
                            imap.insert(
                                openapiv3::StatusCode::Code(422),
                                ReferenceOr::ref_("#/components/responses/command.baddata"),
                            );
                            imap.insert(
                                openapiv3::StatusCode::Code(500),
                                ReferenceOr::ref_("#/components/responses/command.internal"),
                            );

                            imap
                        },
                        ..Default::default()
                    },
                    tags: vec!["commands".to_string()],
                    ..Default::default()
                }),
                ..Default::default()
            }),
        );
    }

    for query in queries.iter() {
        paths.insert(
            format!("/query.{}", query.name),
            ReferenceOr::Item(PathItem {
                get: Some(Operation {
                    tags: vec!["queries".to_string()],
                    responses: Responses {
                        responses: {
                            let mut responses = IndexMap::new();

                            responses.insert(
                                openapiv3::StatusCode::Code(401),
                                ReferenceOr::ref_("#/components/responses/query.unauthorized"),
                            );
                            responses.insert(
                                openapiv3::StatusCode::Code(404),
                                ReferenceOr::ref_("#/components/responses/query.notfound"),
                            );
                            responses.insert(
                                openapiv3::StatusCode::Code(500),
                                ReferenceOr::ref_("#/components/responses/query.internal"),
                            );

                            responses
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                ..Default::default()
            }),
        );
    }

    paths
}

fn generate_responses(
    responses: &mut IndexMap<String, ReferenceOr<Response>>,
    queries: Vec<Query>,
) {
    responses.insert(
        "command.success".to_string(),
        ReferenceOr::Item(Response {
            description: "Server accepted the command".to_string(),
            headers: {
                let mut imap = IndexMap::new();

                imap.insert(
                    "X-Operation-Id".to_string(),
                    ReferenceOr::Item(Header {
                        deprecated: None,
                        description: None,
                        example: None,
                        examples: IndexMap::new(),
                        extensions: IndexMap::new(),
                        format: openapiv3::ParameterSchemaOrContent::Schema(ReferenceOr::Item(
                            Schema {
                                schema_data: SchemaData::default(),
                                schema_kind: SchemaKind::Type(Type::String(StringType::default())),
                            },
                        )),
                        required: false,
                        style: HeaderStyle::Simple,
                    }),
                );

                imap
            },
            ..Default::default()
        }),
    );

    responses.insert(
        "command.unauthorized".to_string(),
        ReferenceOr::Item(Response {
            description: "Server rejected the command, user was not authorized".to_string(),
            ..Default::default()
        }),
    );

    responses.insert(
        "command.baddata".to_string(),
        ReferenceOr::Item(Response {
            description: "Server rejected the command, data was not valid for that endpoint"
                .to_string(),
            ..Default::default()
        }),
    );

    responses.insert(
        "command.internal".to_string(),
        ReferenceOr::Item(Response {
            description: "Server rejected the command, something happened internally".to_string(),
            ..Default::default()
        }),
    );

    responses.insert(
        "query.unauthorized".to_string(),
        ReferenceOr::Item(Response {
            description: "Server rejected the query, user not authorized to see that".to_string(),
            ..Default::default()
        }),
    );

    responses.insert(
        "query.notfound".to_string(),
        ReferenceOr::Item(Response {
            description: "Server rejected the query, requested data not found".to_string(),
            ..Default::default()
        }),
    );

    responses.insert(
        "query.internal".to_string(),
        ReferenceOr::Item(Response {
            description: "Server rejected the query, something happened internally".to_string(),
            ..Default::default()
        }),
    );
}

fn generate_models(schemas: &mut IndexMap<String, ReferenceOr<Schema>>, models: Vec<Model>) {
    for model in models.iter() {
        schemas.insert(model.name.clone(), generate_schema(model));
        schemas.insert(model.name.clone() + "_list", generate_list(model));
    }
}

fn generate_list(model: &Model) -> ReferenceOr<Schema> {
    ReferenceOr::Item(Schema {
        schema_data: SchemaData {
            ..Default::default()
        },
        schema_kind: SchemaKind::Type(Type::Array(ArrayType {
            items: Some(ReferenceOr::ref_(
                (String::from("#/components/schemas/") + model.name.as_str()).as_str(),
            )),
            max_items: None,
            min_items: None,
            unique_items: false,
        })),
    })
}

fn generate_schema(model: &Model) -> ReferenceOr<Schema> {
    let mut required: Vec<String> = Vec::new();
    let mut properties: IndexMap<String, ReferenceOr<Box<Schema>>> = IndexMap::new();

    for property in model.properties.iter() {
        if property.required {
            required.push(property.name.clone());
        }

        let typ = get_schema_type(property.datatype.clone());

        properties.insert(
            property.name.clone(),
            ReferenceOr::Item(Box::new(Schema {
                schema_data: SchemaData {
                    description: {
                        match property.primary {
                            true => Some("Primary Key".to_string()),
                            false => None,
                        }
                    },
                    ..Default::default()
                },
                schema_kind: SchemaKind::Type(*typ),
            })),
        );
    }

    ReferenceOr::Item(Schema {
        schema_kind: SchemaKind::Type(Type::Object(ObjectType {
            properties: properties,
            required: required,
            ..Default::default()
        })),
        schema_data: SchemaData {
            ..Default::default()
        },
    })
}

fn get_schema_type(datatype: DataTypes) -> Box<Type> {
    match datatype {
        DataTypes::ID => {
            let mut st = StringType::default();
            st.pattern = Some(String::from("[0-9A-HJKMNP-TV-Z]{26}"));
            Box::new(Type::String(st))
        }
        DataTypes::String => {
            let st = StringType::default();
            Box::new(Type::String(st))
        }
        DataTypes::Datetime => {
            let mut st = StringType::default();
            st.format = VariantOrUnknownOrEmpty::Item(StringFormat::DateTime);
            Box::new(Type::String(st))
        }
        DataTypes::Pattern(pattern) => {
            let mut st = StringType::default();
            st.pattern = Some(pattern);
            Box::new(Type::String(st))
        }
        DataTypes::Number => Box::new(Type::Number(NumberType::default())),
        DataTypes::Boolean => Box::new(Type::Boolean(BooleanType::default())),
        DataTypes::Model(model) => {
            let mut st = StringType::default();
            st.pattern = Some(format!("{}:[0-9A-HJKMNP-TV-Z]{{26}}", model.clone()));
            Box::new(Type::String(st))
        }
    }
}
