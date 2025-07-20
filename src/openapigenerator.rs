use std::vec;

use indexmap::IndexMap;
use openapiv3::{
    ArrayType, BooleanType, Components, Header, HeaderStyle, Info, License, MediaType, NumberType,
    ObjectType, OpenAPI, Operation, ParameterData, PathItem, Paths, ReferenceOr, RequestBody,
    Response, Responses, Schema, SchemaData, SchemaKind, SecurityScheme, StringFormat, StringType,
    Type, VariantOrUnknownOrEmpty,
};
use parser::{Command, DataTypes, Model, Query, API};

pub fn generate_openapi_spec(api: API) -> OpenAPI {
    let mut schemas: IndexMap<String, ReferenceOr<Schema>> = IndexMap::new();
    let mut responses: IndexMap<String, ReferenceOr<Response>> = IndexMap::new();
    let mut request_bodies: IndexMap<String, ReferenceOr<RequestBody>> = IndexMap::new();
    let mut security_schema: IndexMap<String, ReferenceOr<SecurityScheme>> = IndexMap::new();
    security_schema.insert(
        "bearerAuth".to_string(),
        ReferenceOr::Item(SecurityScheme::HTTP {
            scheme: "bearer".to_string(),
            bearer_format: None,
            description: None,
            extensions: IndexMap::new(),
        }),
    );

    {
        generate_models(&mut schemas, api.models.clone());
        generate_responses(&mut responses, api.queries.clone());
        generate_request_bodies(&mut request_bodies, api.commands.clone());
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
            schemas: schemas.clone(),
            responses: responses.clone(),
            request_bodies: request_bodies.clone(),
            security_schemes: security_schema.clone(),
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
            format!("/command/{}", command.name),
            ReferenceOr::Item(PathItem {
                post: Some(Operation {
                    request_body: Some(ReferenceOr::ref_(
                        ("#/components/requestBodies/command.".to_string() + command.name.as_str())
                            .as_str(),
                    )),
                    parameters: vec![ReferenceOr::Item(openapiv3::Parameter::Query {
                        parameter_data: ParameterData {
                            name: "id".to_string(),
                            required: false,
                            format: openapiv3::ParameterSchemaOrContent::Schema(ReferenceOr::Item(
                                Schema {
                                    schema_data: SchemaData {
                                        ..Default::default()
                                    },
                                    schema_kind: SchemaKind::Type(Type::String(StringType {
                                        pattern: Some("[0-9A-HJKMNP-TV-Z]{26}".to_string()),
                                        ..Default::default()
                                    })),
                                },
                            )),
                            explode: None,
                            deprecated: None,
                            description: None,
                            example: None,
                            examples: IndexMap::new(),
                            extensions: IndexMap::new(),
                        },
                        allow_reserved: false,
                        style: openapiv3::QueryStyle::Form,
                        allow_empty_value: None,
                    })],
                    responses: Responses {
                        responses: {
                            let mut imap = IndexMap::new();

                            imap.insert(
                                openapiv3::StatusCode::Code(202),
                                ReferenceOr::ref_("#/components/responses/command.success"),
                            );
                            if !command.public {
                                imap.insert(
                                    openapiv3::StatusCode::Code(401),
                                    ReferenceOr::ref_(
                                        "#/components/responses/command.unauthorized",
                                    ),
                                );
                            }
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
                    security: if !command.public {
                        Some(vec![IndexMap::from([("bearerAuth".to_string(), vec![])])])
                    } else {
                        None
                    },
                    ..Default::default()
                }),
                ..Default::default()
            }),
        );
    }

    for query in queries.iter() {
        paths.insert(
            format!("/query/{}", query.name),
            ReferenceOr::Item(PathItem {
                get: Some(Operation {
                    parameters: vec![
                        ReferenceOr::Item(openapiv3::Parameter::Query {
                            parameter_data: ParameterData {
                                name: "id".to_string(),
                                required: false,
                                format: openapiv3::ParameterSchemaOrContent::Schema(
                                    ReferenceOr::Item(Schema {
                                        schema_data: SchemaData {
                                            ..Default::default()
                                        },
                                        schema_kind: SchemaKind::Type(Type::String(StringType {
                                            pattern: Some("[0-9A-HJKMNP-TV-Z]{26}".to_string()),
                                            ..Default::default()
                                        })),
                                    }),
                                ),
                                explode: None,
                                deprecated: None,
                                description: None,
                                example: None,
                                examples: IndexMap::new(),
                                extensions: IndexMap::new(),
                            },
                            allow_reserved: false,
                            style: openapiv3::QueryStyle::Form,
                            allow_empty_value: None,
                        }),
                        ReferenceOr::Item(openapiv3::Parameter::Query {
                            parameter_data: ParameterData {
                                name: "page_size".to_string(),
                                required: true,
                                format: openapiv3::ParameterSchemaOrContent::Schema(
                                    ReferenceOr::Item(Schema {
                                        schema_data: SchemaData {
                                            default: Some(serde_json::json!(50)),
                                            ..Default::default()
                                        },
                                        schema_kind: SchemaKind::Type(Type::Number(NumberType {
                                            minimum: Some(1.0),
                                            maximum: Some(100.0),
                                            ..Default::default()
                                        })),
                                    }),
                                ),
                                explode: None,
                                deprecated: None,
                                description: None,
                                example: None,
                                examples: IndexMap::new(),
                                extensions: IndexMap::new(),
                            },
                            allow_reserved: false,
                            style: openapiv3::QueryStyle::Form,
                            allow_empty_value: None,
                        }),
                        ReferenceOr::Item(openapiv3::Parameter::Query {
                            parameter_data: ParameterData {
                                name: "page".to_string(),
                                required: true,
                                format: openapiv3::ParameterSchemaOrContent::Schema(
                                    ReferenceOr::Item(Schema {
                                        schema_data: SchemaData {
                                            default: Some(serde_json::json!(1)),
                                            ..Default::default()
                                        },
                                        schema_kind: SchemaKind::Type(Type::Number(NumberType {
                                            minimum: Some(1.0),
                                            ..Default::default()
                                        })),
                                    }),
                                ),
                                explode: None,
                                deprecated: None,
                                description: None,
                                example: None,
                                examples: IndexMap::new(),
                                extensions: IndexMap::new(),
                            },
                            allow_reserved: false,
                            style: openapiv3::QueryStyle::Form,
                            allow_empty_value: None,
                        }),
                        ReferenceOr::Item(openapiv3::Parameter::Header {
                            parameter_data: ParameterData {
                                name: "If-None-Match".to_string(),
                                required: false,
                                format: openapiv3::ParameterSchemaOrContent::Schema(
                                    ReferenceOr::Item(Schema {
                                        schema_data: SchemaData {
                                            ..Default::default()
                                        },
                                        schema_kind: SchemaKind::Type(Type::String(StringType {
                                            pattern: Some("\"[0-9A-HJKMNP-TV-Z]{26}\"".to_string()),
                                            ..Default::default()
                                        })),
                                    }),
                                ),
                                explode: None,
                                deprecated: None,
                                description: None,
                                example: None,
                                examples: IndexMap::new(),
                                extensions: IndexMap::new(),
                            },
                            style: openapiv3::HeaderStyle::Simple,
                        }),
                    ],
                    tags: vec!["queries".to_string()],
                    responses: Responses {
                        responses: {
                            let mut responses = IndexMap::new();

                            responses.insert(
                                openapiv3::StatusCode::Code(200),
                                ReferenceOr::ref_(
                                    format!("#/components/responses/query.{}.success", query.name)
                                        .as_str(),
                                ),
                            );
                            responses.insert(
                                openapiv3::StatusCode::Code(304),
                                ReferenceOr::Item(Response {
                                    description: "Query not modified".to_string(),
                                    ..Default::default()
                                }),
                            );
                            if !query.public {
                                responses.insert(
                                    openapiv3::StatusCode::Code(401),
                                    ReferenceOr::ref_("#/components/responses/query.unauthorized"),
                                );
                            }
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
                    security: if !query.public {
                        Some(vec![IndexMap::from([("bearerAuth".to_string(), vec![])])])
                    } else {
                        None
                    },
                    ..Default::default()
                }),
                head: Some(Operation {
                    parameters: vec![
                        ReferenceOr::Item(openapiv3::Parameter::Query {
                            parameter_data: ParameterData {
                                name: "id".to_string(),
                                required: false,
                                format: openapiv3::ParameterSchemaOrContent::Schema(
                                    ReferenceOr::Item(Schema {
                                        schema_data: SchemaData {
                                            ..Default::default()
                                        },
                                        schema_kind: SchemaKind::Type(Type::String(StringType {
                                            pattern: Some("[0-9A-HJKMNP-TV-Z]{26}".to_string()),
                                            ..Default::default()
                                        })),
                                    }),
                                ),
                                explode: None,
                                deprecated: None,
                                description: None,
                                example: None,
                                examples: IndexMap::new(),
                                extensions: IndexMap::new(),
                            },
                            allow_reserved: false,
                            style: openapiv3::QueryStyle::Form,
                            allow_empty_value: None,
                        }),
                        ReferenceOr::Item(openapiv3::Parameter::Query {
                            parameter_data: ParameterData {
                                name: "page_size".to_string(),
                                required: true,
                                format: openapiv3::ParameterSchemaOrContent::Schema(
                                    ReferenceOr::Item(Schema {
                                        schema_data: SchemaData {
                                            default: Some(serde_json::json!(50)),
                                            ..Default::default()
                                        },
                                        schema_kind: SchemaKind::Type(Type::Number(NumberType {
                                            minimum: Some(1.0),
                                            maximum: Some(100.0),
                                            ..Default::default()
                                        })),
                                    }),
                                ),
                                explode: None,
                                deprecated: None,
                                description: None,
                                example: None,
                                examples: IndexMap::new(),
                                extensions: IndexMap::new(),
                            },
                            allow_reserved: false,
                            style: openapiv3::QueryStyle::Form,
                            allow_empty_value: None,
                        }),
                        ReferenceOr::Item(openapiv3::Parameter::Query {
                            parameter_data: ParameterData {
                                name: "page".to_string(),
                                required: true,
                                format: openapiv3::ParameterSchemaOrContent::Schema(
                                    ReferenceOr::Item(Schema {
                                        schema_data: SchemaData {
                                            default: Some(serde_json::json!(1)),
                                            ..Default::default()
                                        },
                                        schema_kind: SchemaKind::Type(Type::Number(NumberType {
                                            minimum: Some(1.0),
                                            ..Default::default()
                                        })),
                                    }),
                                ),
                                explode: None,
                                deprecated: None,
                                description: None,
                                example: None,
                                examples: IndexMap::new(),
                                extensions: IndexMap::new(),
                            },
                            allow_reserved: false,
                            style: openapiv3::QueryStyle::Form,
                            allow_empty_value: None,
                        }),
                        ReferenceOr::Item(openapiv3::Parameter::Header {
                            parameter_data: ParameterData {
                                name: "If-None-Match".to_string(),
                                required: false,
                                format: openapiv3::ParameterSchemaOrContent::Schema(
                                    ReferenceOr::Item(Schema {
                                        schema_data: SchemaData {
                                            ..Default::default()
                                        },
                                        schema_kind: SchemaKind::Type(Type::String(StringType {
                                            pattern: Some("\"[0-9A-HJKMNP-TV-Z]{26}\"".to_string()),
                                            ..Default::default()
                                        })),
                                    }),
                                ),
                                explode: None,
                                deprecated: None,
                                description: None,
                                example: None,
                                examples: IndexMap::new(),
                                extensions: IndexMap::new(),
                            },
                            style: openapiv3::HeaderStyle::Simple,
                        }),
                    ],
                    tags: vec!["queries".to_string()],
                    responses: Responses {
                        responses: {
                            let mut responses = IndexMap::new();

                            responses.insert(
                                openapiv3::StatusCode::Code(200),
                                ReferenceOr::Item(Response {
                                    description: "Successful query response".to_string(),
                                    ..Default::default()
                                }),
                            );
                            responses.insert(
                                openapiv3::StatusCode::Code(304),
                                ReferenceOr::Item(Response {
                                    description: "Query not modified".to_string(),
                                    ..Default::default()
                                }),
                            );
                            if !query.public {
                                responses.insert(
                                    openapiv3::StatusCode::Code(401),
                                    ReferenceOr::ref_("#/components/responses/query.unauthorized"),
                                );
                            }
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
                    security: if !query.public {
                        Some(vec![IndexMap::from([("bearerAuth".to_string(), vec![])])])
                    } else {
                        None
                    },
                    ..Default::default()
                }),
                ..Default::default()
            }),
        );
    }

    paths
}

fn generate_request_bodies(
    request_bodies: &mut IndexMap<String, ReferenceOr<RequestBody>>,
    commands: Vec<Command>,
) {
    for command in commands.iter() {
        request_bodies.insert(
            ("command.".to_string() + command.name.as_str()).to_string(),
            ReferenceOr::Item(RequestBody {
                content: {
                    let mut imap = IndexMap::new();

                    let mut mt = MediaType::default();
                    mt.schema = Some(ReferenceOr::ref_(
                        ("#/components/schemas/".to_string() + command.modelled_by.as_str())
                            .as_str(),
                    ));

                    imap.insert("application/json".to_string(), mt.clone());

                    imap
                },
                ..Default::default()
            }),
        );
    }
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

    for query in queries.iter() {
        responses.insert(
            ("query.".to_string() + query.name.as_str() + ".success").to_string(),
            ReferenceOr::Item(Response {
                description: "Sucessful query response".to_string(),
                content: {
                    let mut imap = IndexMap::new();

                    let mut mt = MediaType::default();
                    mt.schema = Some(ReferenceOr::Item(Schema {
                        schema_data: SchemaData::default(),
                        schema_kind: SchemaKind::OneOf {
                            one_of: vec![
                                ReferenceOr::ref_(
                                    ("#/components/schemas/".to_string()
                                        + query.modelled_by.as_str())
                                    .as_str(),
                                ),
                                ReferenceOr::Item(Schema {
                                    schema_data: SchemaData::default(),
                                    schema_kind: SchemaKind::Type(Type::Array(ArrayType {
                                        items: Some(ReferenceOr::ref_(
                                            ("#/components/schemas/".to_string()
                                                + query.modelled_by.as_str())
                                            .as_str(),
                                        )),
                                        max_items: None,
                                        min_items: None,
                                        unique_items: false,
                                    })),
                                }),
                            ],
                        },
                    }));

                    imap.insert("application/json".to_string(), mt.clone());

                    imap
                },
                ..Default::default()
            }),
        );
    }
}

fn generate_models(schemas: &mut IndexMap<String, ReferenceOr<Schema>>, models: Vec<Model>) {
    for model in models.iter() {
        schemas.insert(model.name.clone(), generate_schema(model));
    }
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
