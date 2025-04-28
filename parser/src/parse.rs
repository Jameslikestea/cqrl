use errors::{CQRLError, CQRLResult};
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};

use crate::DataTypes;

impl super::CQRLParser {
    pub fn parse_string(input: &str) -> CQRLResult<crate::API> {
        let tokens = super::CQRLParser::parse(super::Rule::document, input);

        match tokens {
            Ok(pairs) => return super::CQRLParser::process(pairs),
            Err(err) => {
                return Err(CQRLError::LexError {
                    error: err.to_string(),
                })
            }
        }
    }

    fn process(pairs: Pairs<crate::Rule>) -> CQRLResult<crate::API> {
        let mut api = crate::API::new();

        for pair in pairs {
            match process_pair(pair) {
                Ok(a) => {
                    for command in a.commands {
                        api.commands.push(command);
                    }
                    for query in a.queries {
                        api.queries.push(query);
                    }
                    for model in a.models {
                        api.models.push(model);
                    }
                }
                Err(_) => return Err(CQRLError::ParseError),
            }
        }

        return Ok(api);
    }
}

fn process_pair(pair: Pair<crate::Rule>) -> CQRLResult<crate::API> {
    match pair.as_rule() {
        crate::Rule::document => Ok(process_document(pair)),
        _ => Err(CQRLError::ParseError),
    }
}

fn process_document(pair: Pair<crate::Rule>) -> crate::API {
    let mut api = crate::API::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            crate::Rule::stmt => process_stmt(&mut api, inner_pair),
            _ => {}
        }
    }

    api
}

fn process_stmt(api: &mut crate::API, stmt: Pair<crate::Rule>) {
    for inner_type in stmt.into_inner() {
        match inner_type.as_rule() {
            crate::Rule::stmt_model => api.models.push(process_stmt_model(inner_type)),
            crate::Rule::stmt_command => api.commands.push(process_stmt_command(inner_type)),
            crate::Rule::stmt_query => api.queries.push(process_stmt_query(inner_type)),
            _ => {}
        }
    }
}

fn process_stmt_model(stmt: Pair<crate::Rule>) -> crate::Model {
    let mut model = crate::Model {
        name: String::from(""),
        properties: vec![],
    };

    for inner_type in stmt.into_inner() {
        match inner_type.as_rule() {
            crate::Rule::ident => model.name = String::from(inner_type.as_str()),
            crate::Rule::block_model => model.properties = process_block_model(inner_type),
            _ => {}
        }
    }

    model
}

fn process_block_model(stmt: Pair<crate::Rule>) -> Vec<crate::ModelProperty> {
    let mut properties: Vec<crate::ModelProperty> = Vec::new();

    for property in stmt.into_inner() {
        match property.as_rule() {
            crate::Rule::model_property => match process_model_property(property) {
                Ok(property) => properties.push(property),
                Err(_) => {}
            },
            rule => println!("{:#?}", rule),
        }
    }

    properties
}

fn process_model_property(stmt: Pair<crate::Rule>) -> CQRLResult<crate::ModelProperty> {
    let mut property = crate::ModelProperty {
        name: "".to_string(),
        datatype: crate::DataTypes::ID,
        required: false,
        primary: false,
        list: false,
    };

    for token in stmt.into_inner() {
        match token.as_rule() {
            crate::Rule::ident => property.name = token.as_str().to_string(),
            crate::Rule::dt_list => {
                let dt = token
                    .into_inner()
                    .next()
                    .expect("Parsing went horribly wrong!");
                property.list = true;
                property.datatype = process_dt(dt);
            }
            crate::Rule::dt_basic => property.datatype = process_dt(token),
            crate::Rule::kw_required => property.required = true,
            crate::Rule::kw_primary => property.primary = true,
            _ => {}
        }
    }

    Ok(property)
}

fn process_dt(stmt: Pair<crate::Rule>) -> DataTypes {
    let dt = stmt
        .into_inner()
        .next()
        .expect("Parsing went horribly wrong!");

    match dt.as_rule() {
        crate::Rule::dt_id => DataTypes::ID,
        crate::Rule::dt_string => DataTypes::String,
        // crate::Rule::dt_pattern => {
        //     println!("Enter Pattern");
        //     DataTypes::Pattern("[0-9]".to_string())
        // }
        crate::Rule::dt_datetime => DataTypes::Datetime,
        crate::Rule::dt_number => DataTypes::Number,
        crate::Rule::dt_boolean => DataTypes::Boolean,
        crate::Rule::ident => DataTypes::Model(dt.as_str().to_string()),
        _ => DataTypes::String,
    }
}

fn process_stmt_command(stmt: Pair<crate::Rule>) -> crate::Command {
    let mut command = crate::Command {
        name: String::from(""),
        modelled_by: String::from(""),
    };

    for inner_type in stmt.into_inner() {
        match inner_type.as_rule() {
            crate::Rule::ident => command.name = String::from(inner_type.as_str()),
            crate::Rule::block_command => command.modelled_by = process_block_command(inner_type),
            _ => {}
        }
    }

    command
}

fn process_block_command(stmt: Pair<crate::Rule>) -> String {
    let modelled_by = stmt
        .into_inner()
        .next()
        .expect("required property modelled by");

    for inner in modelled_by.into_inner() {
        match inner.as_rule() {
            crate::Rule::ident => return inner.as_str().to_string(),
            _ => {}
        }
    }

    String::from("")
}

fn process_stmt_query(stmt: Pair<crate::Rule>) -> crate::Query {
    let mut query = crate::Query {
        name: String::from(""),
        modelled_by: String::from(""),
    };

    for inner_type in stmt.into_inner() {
        match inner_type.as_rule() {
            crate::Rule::ident => query.name = String::from(inner_type.as_str()),
            crate::Rule::block_query => query.modelled_by = process_block_query(inner_type),
            _ => {}
        }
    }

    query
}

fn process_block_query(stmt: Pair<crate::Rule>) -> String {
    let modelled_by = stmt
        .into_inner()
        .next()
        .expect("required property modelled by");

    for inner in modelled_by.into_inner() {
        match inner.as_rule() {
            crate::Rule::ident => return inner.as_str().to_string(),
            _ => {}
        }
    }

    String::from("")
}
