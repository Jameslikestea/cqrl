use pest::{iterators::Pairs, Parser};

pub fn lex_tokens(input: &str) -> Pairs<crate::Rule> {
    let output = crate::CQRLParser::parse(crate::Rule::document, input);
    match output {
        Ok(pairs) => return pairs,
        Err(err) => panic!("Cannot parse: {:?}", err),
    }
}
