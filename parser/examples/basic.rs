use std::fs;

pub fn main() {
    let input = fs::read_to_string("./examples/basic.cqrl").unwrap();
    let lexed = parser::CQRLParser::parse_string(input.as_str());
    match lexed {
        Ok(api) => match serde_json::to_string_pretty(&api) {
            Ok(out) => {
                println!("{}", out)
            }
            Err(err) => println!("{}", err.to_string()),
        },
        Err(err) => println!("{}", err.to_string()),
    }
}
