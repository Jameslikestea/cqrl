use super::*;

#[test]
fn empty_string() {
    let input = "";
    let output = lexer::lex_tokens(input);

    assert_eq!(output.as_str(), "");
}

#[test]
fn basic_command() {}

#[test]
fn basic_query() {}

#[test]
fn basic_model() {
    let input = "model \"test\" { id: id required primary name : string enabled : boolean}";
    let output = lexer::lex_tokens(input);

    assert_eq!(output.as_str(), input)
}
