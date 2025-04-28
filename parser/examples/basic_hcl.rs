use std::fs;

use parser::{parse_hcl, validate::validate_api, API};

fn main() {
    let input = fs::read_to_string("./examples/basic.hcl").unwrap();
    let parsed: API = parse_hcl::parse_hcl(&input).expect("Failed to parse HCL");
    match validate_api(parsed) {
        Ok(api) => match serde_json::to_string_pretty(&api) {
            Ok(out) => {
                println!("{}", out);
            }
            Err(err) => println!("{}", err.to_string()),
        },
        Err(err) => println!("{}", err.to_string()),
    }
}
