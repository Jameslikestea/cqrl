use std::fs;

use parser::Rule;
use pest::iterators::Pair;

pub fn main() {
    let input = fs::read_to_string("./examples/basic.cqrl").unwrap();
    let lexed = parser::lexer::lex_tokens(input.as_str());
    for pair in lexed {
        process_pair(pair, 0);
    }
}

fn process_pair(pair: Pair<Rule>, indent: usize) {
    println!(
        "{:indent$}> {:#?}",
        " ",
        pair.as_rule(),
        indent = indent * 2
    );
    for inner_pair in pair.into_inner() {
        process_pair(inner_pair, indent + 1);
    }
}
