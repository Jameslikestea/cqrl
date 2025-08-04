use errors::CQRLResult;
use hcl::{eval::Evaluate, Block, Body, Value};
use indexmap::IndexMap;

pub fn parse_hcl(input: &str) -> CQRLResult<crate::API> {
    let parsed: Body = hcl::from_str(&input).expect("Failed to parse");
    let mut ctx = hcl::eval::Context::new();

    let mut api = crate::API::new();
    let mut commands = Vec::new();
    let mut queries = Vec::new();
    let mut models = Vec::new(); // Store parsed models

    let mut context_commands = IndexMap::new();
    let mut context_queries = IndexMap::new();
    let mut context_models = IndexMap::new();

    for block in parsed.blocks() {
        match block.identifier() {
            "command" => match block.labels().first() {
                Some(label) => {
                    context_commands.insert(
                        label.as_str().to_string(),
                        Value::String(label.as_str().to_string()),
                    );
                }
                None => {
                    println!("command: none");
                }
            },
            "query" => match block.labels().first() {
                Some(label) => {
                    context_queries.insert(
                        label.as_str().to_string(),
                        Value::String(label.as_str().to_string()),
                    );
                }
                None => {}
            },
            "model" => match block.labels().first() {
                Some(label) => {
                    context_models.insert(
                        label.as_str().to_string(),
                        Value::String(label.as_str().to_string()),
                    );
                }
                None => {}
            },
            _ => {}
        }
    }

    ctx.declare_var("command", Value::Object(context_commands));
    ctx.declare_var("query", Value::Object(context_queries));
    ctx.declare_var("model", Value::Object(context_models));

    for block in parsed.blocks() {
        match block.identifier() {
            "command" => {
                let command = parse_command(block, &ctx);
                commands.push(command);
            }
            "query" => {
                let query = parse_query(block, &ctx);
                queries.push(query);
            }
            "model" => {
                let model = parse_model(block, &ctx); // Parse the model
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

fn parse_command(block: &Block, ctx: &hcl::eval::Context) -> crate::Command {
    let mut modelled_by = String::from("model");
    let mut authorized_by = String::from("query");
    let mut public = false;

    for attribute in block.body().attributes() {
        if attribute.key() == "modelled_by" {
            let expr = attribute.expr().evaluate(ctx).unwrap();
            modelled_by = expr.as_str().unwrap().to_string();
        }

        if attribute.key() == "public" {
            let expr = attribute.expr().evaluate(ctx).unwrap();
            public = match expr.as_bool() {
                Some(b) => b,
                None => false,
            };
        }

        if attribute.key() == "authorized_by" {
            let expr = attribute.expr().evaluate(ctx).unwrap();
            authorized_by = expr.as_str().unwrap().to_string();
        }
    }

    crate::Command {
        name: block.labels().first().unwrap().as_str().to_string(),
        modelled_by,
        authorized_by,
        public,
    }
}

fn parse_query(block: &Block, ctx: &hcl::eval::Context) -> crate::Query {
    let mut modelled_by = String::from("model");
    let mut public = false;

    for attribute in block.body().attributes() {
        if attribute.key() == "modelled_by" {
            let expr = attribute.expr().evaluate(ctx).unwrap();
            modelled_by = expr.as_str().unwrap().to_string();
        }

        if attribute.key() == "public" {
            let expr = attribute.expr().evaluate(ctx).unwrap();
            public = match expr.as_bool() {
                Some(b) => b,
                None => false,
            };
        }
    }

    crate::Query {
        name: block.labels().first().unwrap().as_str().to_string(),
        modelled_by,
        public,
    }
}

fn parse_model(block: &Block, _ctx: &hcl::eval::Context) -> crate::Model {
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
