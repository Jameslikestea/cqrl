use errors::{CQRLError, CQRLResult};

use crate::DataTypes;

pub fn validate_api(api: crate::API) -> CQRLResult<crate::API> {
    let models = generate_model_map(api.clone());

    if models.contains(&"test".to_string()) {
        println!("contains test");
    }

    if !validate_models(models.clone(), api.models.clone()) {
        return Err(CQRLError::ModelTypes);
    }

    if !validate_commands(models.clone(), api.commands.clone()) {
        return Err(CQRLError::CommandTypes);
    }

    if !validate_queries(models.clone(), api.queries.clone()) {
        return Err(CQRLError::QueryTypes);
    }

    Ok(api)
}

pub fn generate_model_map(api: crate::API) -> Vec<String> {
    let mut models = Vec::new();

    api.models.iter().for_each(|m| {
        models.push(m.name.clone());
    });

    models
}

fn validate_models(models: Vec<String>, api: Vec<crate::Model>) -> bool {
    api.iter()
        .map(|model| {
            model
                .properties
                .iter()
                .map(|f| match f.datatype.clone() {
                    DataTypes::Model(model) => {
                        println!("{:#?}, {}", models, model);
                        models.iter().any(|a| a.eq(&model))
                    }
                    _ => true,
                })
                .reduce(|acc, f| acc && f)
        })
        .reduce(|acc, f| Some(acc.expect("no accumulator") && f.expect("no new value")))
        .expect("cannot reduce API")
        .expect("cannot reduce API")
}

fn validate_commands(models: Vec<String>, commands: Vec<crate::Command>) -> bool {
    commands
        .iter()
        .map(|command| models.contains(&command.modelled_by))
        .reduce(|acc, f| acc && f)
        .expect("Cannot reduce commands")
}

fn validate_queries(models: Vec<String>, queries: Vec<crate::Query>) -> bool {
    queries
        .iter()
        .map(|query| models.contains(&query.modelled_by))
        .reduce(|acc, q| acc && q)
        .expect("Cannot reduce queries")
}
