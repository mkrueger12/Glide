#![deny(warnings)]
//#![allow(dead_code)]

//use std::collections::HashMap;
//use serde_json::Value;
use crate::config::settings::CONF;
use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Payload {
    // This comes from the client
    pub model: Vec<String>,
    pub prompt: Vec<String>,
}

pub struct FirstOption {
    pub model: String,
    pub prompt: String,
    pub provider: String,
    //messages: HashMap<String, Value>,
    //parameters: String,
}
pub struct ScndOption {
    pub model: String,
    pub prompt: String,
    pub provider: String,
    //messages: HashMap<String, Value>,
    //parameters: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIStatusApiResponse {
    status: OpenAIStatus,
}

#[derive(Debug, Deserialize)]
struct OpenAIStatus {
    //description: String,
    indicator: String,
}

pub enum ProviderOptions {
    First(FirstOption),
    Second(ScndOption),
}

pub async fn model_route(
    payload: Payload,
) -> Result<ProviderOptions, Box<dyn Error + Send + Sync>> {
    let (first_option, scnd_option) = parse_post(payload).await;

    let first_option_provider = first_option.provider.clone();
    let scnd_option_provider = scnd_option.provider.clone();

    match check_api_status(first_option_provider).await {
        Ok(status) if status == "OK" => Ok(ProviderOptions::First(first_option)),
        _ => match check_api_status(scnd_option_provider).await {
            Ok(status) if status == "OK" => Ok(ProviderOptions::Second(scnd_option)),
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Both APIs are down",
            ))),
        },
    }
}

pub async fn check_api_status(provider: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    let openai_status: &String = CONF
        .as_ref()
        .map(|settings| &settings.openai.status)
        .unwrap();
    let cohere_status: &String = CONF
        .as_ref()
        .map(|settings| &settings.openai.status)
        .unwrap();

    if provider == "openai" {
        let response: OpenAIStatusApiResponse = reqwest::get(openai_status) //
            .await?
            .json()
            .await?;
        #[cfg(test)] // only print this in tests
        print!("{:#?}", response);
        let status = response.status.indicator; // "none", "minor", "major", "critical"

        if status != "none" {
            eprintln!("OpenAI API Status: {}", status);
            let io_error = std::io::Error::new(std::io::ErrorKind::Other, "OpenAI API is down");
            Err(Box::new(io_error) as Box<dyn std::error::Error + Send + Sync>)
        } else {
            println!("OpenAI API is Operational");
            Ok("OK".to_string())
        }
    } else if provider == "cohere" {
        let response: OpenAIStatusApiResponse = reqwest::get(cohere_status) // TODO: use CONF.openai.status_endpoint
            .await?
            .json()
            .await?;
        #[cfg(test)] // only print this in tests
        print!("{:#?}", response);
        let status = response.status.indicator; // "none", "minor", "major", "critical"

        if status != "none" {
            eprintln!("Cohere API Status: {}", status);
            let io_error = std::io::Error::new(std::io::ErrorKind::Other, "Cohere API is down");
            Err(Box::new(io_error) as Box<dyn std::error::Error + Send + Sync>)
        } else {
            println!("Cohere API is Operational");
            Ok("OK".to_string())
        }
    } else {
        let io_error =
            std::io::Error::new(std::io::ErrorKind::Other, "LLM provider not yet supported.");
        //println!("Unknown provider");
        println!("{:#?}", io_error);
        Err(Box::new(io_error) as Box<dyn std::error::Error + Send + Sync>)
    }
}

fn get_provider(model: &str) -> String {
    let openai_models: &Vec<String> = CONF
        .as_ref()
        .map(|settings| &settings.openai.models)
        .unwrap();
    let cohere_models: &Vec<String> = CONF
        .as_ref()
        .map(|settings| &settings.cohere.models)
        .unwrap();

    let model_string = model.to_string();

    if openai_models.contains(&model_string) {
        "openai".to_string()
    } else if cohere_models.contains(&model_string) {
        "cohere".to_string()
    } else {
        "none".to_string()
    }
}

pub async fn parse_post(payload: Payload) -> (FirstOption, ScndOption) {
    let first_option = FirstOption {
        model: payload.model.get(0).unwrap_or(&String::new()).clone(),
        prompt: payload.prompt.get(0).unwrap_or(&String::new()).clone(),
        provider: get_provider(payload.model.get(0).unwrap_or(&String::new())),
        //messages: payload.messages.get(0).unwrap_or(&HashMap::new()).clone(),
        // parameters: payload.parameters.get(0).unwrap_or(&String::new()).clone(),
    };

    let scnd_option = ScndOption {
        model: payload.model.get(1).unwrap_or(&String::new()).clone(),
        prompt: payload.prompt.get(1).unwrap_or(&String::new()).clone(),
        provider: get_provider(payload.model.get(1).unwrap_or(&String::new())),
        // messages: payload.messages.get(0).unwrap_or(&HashMap::new()).clone(),
        //parameters: payload.parameters.get(1).unwrap_or(&String::new()).clone(),
    };

    (first_option, scnd_option)
}

// ###### TESTS ######

#[cfg(test)]
mod tests {
    use crate::handlers::model_router::check_api_status;
    use tokio;

    #[tokio::test]
    async fn test_check_api_status() {
        // Check if OpenAI API is up
        let openai_status = check_api_status("openai".to_string()).await;
        // assert_eq!(openai_status, "OK");
        match openai_status {
            Ok(status) => assert_eq!(status, "OK"),
            Err(e) => assert!(e.to_string().contains("OpenAI API is down")),
        }

        // Check if Anthropic API is up
        let anthropic_status = check_api_status("anthropic".to_string()).await.unwrap();
        assert_eq!(anthropic_status, "Anthropic API is Operational");

        // Check if unknown API is up
        let unknown_status = check_api_status("unknown".to_string()).await;
        assert!(unknown_status.is_err());
    }
}
