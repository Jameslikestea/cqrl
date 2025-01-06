use pest_derive::Parser;

pub mod lexer;

#[derive(Parser)]
#[grammar = "cqrl.pest"]
pub struct CQRLParser;
