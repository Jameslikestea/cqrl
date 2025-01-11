use std::fs;

use parser::validate::validate_api;

pub fn main() {
    let input = fs::read_to_string("./examples/basic.cqrl").unwrap();
    let lexed = parser::CQRLParser::parse_string(input.as_str());
    match validate_api(lexed.expect("Expect deserialize")) {
        Ok(api) => match serde_json::to_string_pretty(&api) {
            Ok(out) => {
                println!("{}", out);
            }
            Err(err) => println!("{}", err.to_string()),
        },
        Err(err) => println!("{}", err.to_string()),
    }
}
