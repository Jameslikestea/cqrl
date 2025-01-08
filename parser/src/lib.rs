use pest_derive::Parser;
use serde::{Deserialize, Serialize};

pub mod lexer;
pub mod parse;

#[derive(Debug, Parser)]
#[grammar = "cqrl.pest"]
pub struct CQRLParser;

#[derive(Debug, Serialize, Deserialize)]
pub struct API {
    pub commands: Vec<Command>,
    pub queries: Vec<Query>,
    pub models: Vec<Model>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    name: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Query {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DataTypes {
    ID,
    String,
    Pattern,
    Datetime,
    Number,
    Boolean,
    Model(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelProperty {
    name: String,
    datatype: DataTypes,
    required: bool,
    primary: bool,
    list: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    name: String,
    properties: Vec<ModelProperty>,
}

impl API {
    pub fn new() -> Self {
        API {
            commands: Vec::new(),
            models: Vec::new(),
            queries: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests;
