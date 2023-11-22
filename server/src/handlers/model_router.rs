#![deny(warnings)]

use serde::{Deserialize, Serialize};

use warp::Filter;

#[derive(Deserialize, Serialize)]
pub struct Payload {
    model: Vec<String>,
    prompt: Vec<String>,
    messages: Vec<String>,
    parameters: Vec<String>,
}

pub struct FirstOption {
    model: String,
    prompt: String,
    messages: String,
    parameters: String,
}
pub struct ScndOption {
    model: String,
    prompt: String,
    messages: String,
    parameters: String,
}

#[tokio::main]
pub async fn model_router(payload: Payload) -> (FirstOption, ScndOption) {
    let first_option = FirstOption {
        model: payload.model.get(0).unwrap_or(&String::new()).clone(),
        prompt: payload.prompt.get(0).unwrap_or(&String::new()).clone(),
        messages: payload.messages.get(0).unwrap_or(&String::new()).clone(),
        parameters: payload.parameters.get(0).unwrap_or(&String::new()).clone(),
    };

    let scnd_option = ScndOption {
        model: payload.model.get(1).unwrap_or(&String::new()).clone(),
        prompt: payload.prompt.get(1).unwrap_or(&String::new()).clone(),
        messages: payload.messages.get(1).unwrap_or(&String::new()).clone(),
        parameters: payload.parameters.get(1).unwrap_or(&String::new()).clone(),
    };

    (first_option, scnd_option)
}