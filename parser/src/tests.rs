use super::*;

#[test]
fn empty_string() {
    let input = "";
    let output = lexer::lex_tokens(input);

    assert_eq!(output.as_str(), "");
}
