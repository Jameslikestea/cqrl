use validate::validate_api;

use super::*;

#[test]
fn empty_string() {
    let input = "";
    let output = lexer::lex_tokens(input);

    assert_eq!(output.as_str(), "");
}

#[test]
fn basic_model() {
    let input = "model \"test\" { id: id required primary name : string enabled : boolean}";
    let output = lexer::lex_tokens(input);

    assert_eq!(output.as_str(), input)
}

#[test]
fn test_validate_api() {
    let input = r#"
    command "test" {
        modelled_by: test_input
    }

    query "test" {
        modelled_by: test_output
    }

    model "test_input" {
        name: string
        age: number
    }

    model "test_output" {
        id: id
        name: string
        age: number
    }
    "#;

    let output = CQRLParser::parse_string(input).expect("input should parse");

    let valid = validate_api(output).expect("api should be valid");

    assert_eq!(
        valid,
        API {
            commands: vec![Command {
                name: "test".to_string(),
                modelled_by: "test_input".to_string(),
                public: false,
            }],
            queries: vec![Query {
                name: "test".to_string(),
                modelled_by: "test_output".to_string(),
                public: false,
            }],
            models: vec![
                Model {
                    name: "test_input".to_string(),
                    properties: vec![
                        ModelProperty {
                            name: "name".to_string(),
                            datatype: DataTypes::String,
                            list: false,
                            primary: false,
                            required: false,
                        },
                        ModelProperty {
                            name: "age".to_string(),
                            datatype: DataTypes::Number,
                            list: false,
                            primary: false,
                            required: false,
                        }
                    ]
                },
                Model {
                    name: "test_output".to_string(),
                    properties: vec![
                        ModelProperty {
                            name: "id".to_string(),
                            datatype: DataTypes::ID,
                            list: false,
                            primary: false,
                            required: false,
                        },
                        ModelProperty {
                            name: "name".to_string(),
                            datatype: DataTypes::String,
                            list: false,
                            primary: false,
                            required: false,
                        },
                        ModelProperty {
                            name: "age".to_string(),
                            datatype: DataTypes::Number,
                            list: false,
                            primary: false,
                            required: false,
                        }
                    ]
                }
            ],
        },
    );
}
