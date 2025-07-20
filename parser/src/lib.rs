use std::fmt::Display;

use pest_derive::Parser;
use serde::{Deserialize, Serialize};

pub mod lexer;
pub mod parse;
pub mod parse_hcl;
pub mod validate;

#[derive(Debug, Parser)]
#[grammar = "cqrl.pest"]
pub struct CQRLParser;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct API {
    pub commands: Vec<Command>,
    pub queries: Vec<Query>,
    pub models: Vec<Model>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Command {
    pub name: String,
    pub modelled_by: String,
    pub public: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Query {
    pub name: String,
    pub modelled_by: String,
    pub public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataTypes {
    ID,
    String,
    Pattern(String),
    Datetime,
    Number,
    Boolean,
    Model(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelProperty {
    pub name: String,
    pub datatype: DataTypes,
    pub required: bool,
    pub primary: bool,
    pub list: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Model {
    pub name: String,
    pub properties: Vec<ModelProperty>,
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

impl Display for DataTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.clone() {
            DataTypes::Boolean => write!(f, "Boolean"),
            DataTypes::Datetime => write!(f, "Datetime"),
            DataTypes::ID => write!(f, "ID"),
            DataTypes::Model(model) => write!(f, "ID -> {}", model),
            DataTypes::Number => write!(f, "Number"),
            DataTypes::Pattern(pattern) => write!(f, "String ({})", pattern),
            DataTypes::String => write!(f, "String"),
        }
    }
}

#[cfg(test)]
mod tests;
