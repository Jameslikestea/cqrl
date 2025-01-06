fn main() {
    for pair in parser::lexer::lex_tokens("") {
        println!("{:#?}", pair.as_rule())
    }
}
