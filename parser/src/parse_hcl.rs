use errors::CQRLResult;
use hcl::{Block, Body};

pub fn parse_hcl(input: &str) -> CQRLResult<crate::API> {
    let parsed: Body = hcl::from_str(&input).expect("Failed to parse");
    let mut api = crate::API::new();
    let mut commands = Vec::new();
    let mut queries = Vec::new();
    let mut models = Vec::new(); // Store parsed models

    for block in parsed.blocks() {
        match block.identifier() {
            "command" => {
                let command = parse_command(block);
                commands.push(command);
            }
            "query" => {
                let query = parse_query(block);
                queries.push(query);
            }
            "model" => {
                let model = parse_model(block); // Parse the model
                models.push(model); // Add model to the list
            }
            _ => {}
        }
    }

    api.commands = commands;
    api.queries = queries;
    api.models = models; // Assuming API has a models field

    Ok(api)
}

fn parse_command(block: &Block) -> crate::Command {
    let mut modelled_by = String::from("model");

    for attribute in block.body().attributes() {
        if attribute.key() == "modelled_by" {
            let expr = attribute.expr().to_string();
            modelled_by = expr.to_string();
            break;
        }
    }

    if modelled_by.starts_with("model.") {
        modelled_by = modelled_by.strip_prefix("model.").unwrap().to_string();
    }

    crate::Command {
        name: block.labels().first().unwrap().as_str().to_string(),
        modelled_by,
    }
}

fn parse_query(block: &Block) -> crate::Query {
    let mut modelled_by = String::from("model");

    for attribute in block.body().attributes() {
        if attribute.key() == "modelled_by" {
            let expr = attribute.expr().to_string();
            modelled_by = expr.to_string();
            break;
        }
    }

    if modelled_by.starts_with("model.") {
        modelled_by = modelled_by.strip_prefix("model.").unwrap().to_string();
    }

    crate::Query {
        name: block.labels().first().unwrap().as_str().to_string(),
        modelled_by,
    }
}

fn parse_model(block: &Block) -> crate::Model {
    let mut properties = Vec::new();

    for attribute in block.body().attributes() {
        let property_name = attribute.key().to_string();
        let property_block = match attribute.expr() {
            hcl::Expression::Object(obj) => obj,
            _ => continue,
        };

        // Extract the type field and handle Identifier
        let type_str = property_block
            .get(&hcl::ObjectKey::from(hcl::Identifier::from("type")))
            .and_then(|v| match v {
                hcl::Expression::String(s) => Some(s.to_string()),
                _ => None,
            })
            .unwrap_or_else(|| "string".to_string());

        let datatype = match type_str.as_str() {
            "id" => crate::DataTypes::ID,
            "string" => crate::DataTypes::String,
            "datetime" => crate::DataTypes::Datetime,
            "number" => crate::DataTypes::Number,
            "boolean" => crate::DataTypes::Boolean,
            _ => crate::DataTypes::String,
        };

        // Similarly handle boolean values which might be identifiers
        let required = property_block
            .get(&hcl::ObjectKey::from(hcl::Identifier::from("required")))
            .and_then(|v| match v {
                hcl::Expression::Bool(b) => Some(*b),
                hcl::Expression::String(id) => Some(id == "true"),
                _ => None,
            })
            .unwrap_or(false);

        let primary = property_block
            .get(&hcl::ObjectKey::from(hcl::Identifier::from("primary")))
            .and_then(|v| match v {
                hcl::Expression::Bool(b) => Some(*b),
                hcl::Expression::String(id) => Some(id == "true"),
                _ => None,
            })
            .unwrap_or(false);

        let list = property_block
            .get(&hcl::ObjectKey::from(hcl::Identifier::from("list")))
            .and_then(|v| match v {
                hcl::Expression::Bool(b) => Some(*b),
                hcl::Expression::String(id) => Some(id == "true"),
                _ => None,
            })
            .unwrap_or(false);

        properties.push(crate::ModelProperty {
            name: property_name,
            datatype,
            required,
            primary,
            list,
        });
    }

    crate::Model {
        name: block.labels().first().unwrap().as_str().to_string(),
        properties,
    }
}
