#![deny(warnings)]

use std::{collections::HashMap};
use serde_json::Value;

use serde::{Deserialize, Serialize};

// use warp::Filter;

#[derive(Deserialize, Serialize)]
pub struct Payload {
    model: Vec<String>,
    prompt: Vec<String>,
    messages: Vec<HashMap<String, Value>>,
    parameters: Vec<String>,
}

pub struct FirstOption {
    model: String,
    prompt: String,
    messages: HashMap<String, Value>,
    parameters: String,
}
pub struct ScndOption {
    model: String,
    prompt: String,
    messages: HashMap<String, Value>,
    parameters: String,
}

pub async fn check_api_status(provider: String) -> String {

    if provider == "openai" {
        let status = curl https://status.openai.com/api/v2/summary.json
        status
    } else if provider == "anthropic" {
        let status = curl https://status.openai.com/api/v2/summary.json
        status
    } else {
        let status = "none".to_string()
        status
    }

    let open_ai_status = curl https://status.openai.com/api/v2/summary.json
    status
}


fn get_provider(model: &str) -> String {
    if ["gpt3", "gpt4"].contains(&model) {
        "openai".to_string()
    } else if ["claude-instant-1.2", "claude-2.1"].contains(&model) {
        "anthropic".to_string()
    } else {
        "none".to_string()
    }
}

pub async fn select_model(first_option: FirstOption, scnd_options: ScndOption) -> (String, String) {
    let first_option_provider = get_provider(&first_option.model);
    let scnd_option_provider = get_provider(&scnd_options.model);

    (first_option_provider, scnd_option_provider)
}

pub async fn parse_post(payload: Payload) -> (FirstOption, ScndOption) {
    let first_option = FirstOption {
        model: payload.model.get(0).unwrap_or(&String::new()).clone(),
        prompt: payload.prompt.get(0).unwrap_or(&String::new()).clone(),
        messages: payload.messages.get(0).unwrap_or(&HashMap::new()).clone(),
        parameters: payload.parameters.get(0).unwrap_or(&String::new()).clone(),
    };

    let scnd_option = ScndOption {
        model: payload.model.get(1).unwrap_or(&String::new()).clone(),
        prompt: payload.prompt.get(1).unwrap_or(&String::new()).clone(),
        messages: payload.messages.get(0).unwrap_or(&HashMap::new()).clone(),
        parameters: payload.parameters.get(1).unwrap_or(&String::new()).clone(),
    };

    (first_option, scnd_option)
}