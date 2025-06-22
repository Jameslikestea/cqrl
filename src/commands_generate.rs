use std::{error::Error, fs};

use clap::Subcommand;

use parser::{parse_hcl::parse_hcl, API};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Subcommand, Deserialize, Serialize)]
pub(crate) enum GenerateCommand {
    Openapi {
        #[arg(required(true), value_name("SERVICE_FILE"), help = "The service file to generate the OpenAPI spec for")]
        input: String,
        #[arg(required(true), value_name("OUTPUT_FILE"), help = "The output file to write the OpenAPI spec to")]
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

    match parse_hcl(&content) {
        Ok(parsed_api) => {
            api = parsed_api;
        }
        Err(err) => {
            println!("Cannot parse input file: `{}`: {}", input, err);
        }
    };

    let openapi = crate::openapigenerator::generate_openapi_spec(api);
    // println!("Parsed input: {:#?}", openapi);
    let openapi_string = serde_json::to_string_pretty(&openapi).unwrap();
    fs::write(output, openapi_string).unwrap();
}
