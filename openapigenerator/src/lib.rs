use indexmap::IndexMap;
use openapiv3::{
    ArrayType, BooleanType, Components, Info, License, NumberType, ObjectType, OpenAPI,
    ReferenceOr, Schema, SchemaData, SchemaKind, StringFormat, StringType, Type,
    VariantOrUnknownOrEmpty,
};
use parser::{DataTypes, Model, API};

pub fn generate_openapi_spec(api: API) -> OpenAPI {
    let mut schemas: IndexMap<String, ReferenceOr<Schema>> = IndexMap::new();

    {
        generate_models(&mut schemas, api.models);
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
        components: Some(Components {
            schemas: schemas,
            ..Default::default()
        }),
        ..Default::default()
    }
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
