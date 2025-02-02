use std::{error::Error, fs};

use clap::Subcommand;

use parser::{CQRLParser, API};

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum GenerateCommand {
    Openapi {
        #[arg(long, short, default_value_t = String::from("./service.cqrl"))]
        input: String,
        #[arg(long, short, default_value_t = String::from("./service.openapi.json"))]
        output: String,
    },
}

impl GenerateCommand {
    pub(crate) async fn run(self: Self) -> Result<(), Box<dyn Error>> {
        match self {
            GenerateCommand::Openapi { input, output } => {
                generate_openapi(input, output);
                Ok(())
            }
        }
    }
}

fn generate_openapi(input: String, output: String) {
    {
        println!("Generating OpenAPI Spec for {}", input);
    }
    let mut content: String = String::new();

    match fs::read_to_string(input.clone()) {
        Ok(file) => content = file,
        Err(err) => {
            println!("Cannot read input file `{}`: {}", input, err);
        }
    };

    let mut api: API = API::new();

    match CQRLParser::parse_string(&content) {
        Ok(parsed_api) => {
            api = parsed_api;
        }
        Err(err) => {
            println!("Cannot parse input file: `{}`: {}", input, err);
        }
    };

    let openapi = openapigenerator::generate_openapi_spec(api);
    // println!("Parsed input: {:#?}", openapi);
    let openapi_string = serde_json::to_string_pretty(&openapi).unwrap();
    fs::write(output, openapi_string).unwrap();
}
