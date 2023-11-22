#![deny(warnings)]
#![allow(dead_code)]

//use std::collections::HashMap;
//use serde_json::Value;
use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

// use warp::Filter;

#[derive(Deserialize, Serialize)]
pub struct Payload { // This comes from the client
    pub model: Vec<String>,
    pub prompt: Vec<String>,
    //messages: Vec<HashMap<String, Value>>,
    //parameters: Vec<String>,
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
    description: String,
    indicator: String,
}

pub enum ProviderOptions {
    First(FirstOption),
    Second(ScndOption),
}

pub async fn model_route(payload: Payload) -> Result<ProviderOptions, Box<dyn Error + Send + Sync>> {
    // Parse the POST payload
    let (first_option, scnd_option) = parse_post(payload).await;

    // Select the model
    let first_option_provider = first_option.provider.clone();
    let scnd_option_provider = scnd_option.provider.clone();

    // Check the API status for the first option provider
    match check_api_status(first_option_provider).await {
        Ok(status) if status == "OK" => {
            println!("First option provider API is Operational");
            // Continue with the rest of your code for the first option...
            return Ok(ProviderOptions::First(first_option));
        },
        _ => {
            // If the first option API is down, check the second option
            match check_api_status(scnd_option_provider).await {
                Ok(status) if status == "OK" => {
                    println!("Second option provider API is Operational");
                    // Continue with the rest of your code for the second option...
                    return Ok(ProviderOptions::Second(scnd_option));
                },
                _ => {
                    return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Both APIs are down")));
                }
            }
        }
    }
}

pub async fn check_api_status(provider: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    if provider == "openai" {
        let response: OpenAIStatusApiResponse = reqwest::get("https://status.openai.com/api/v2/summary.json").await?.json().await?;
        #[cfg(test)] // only print this in tests
        print!("{:#?}", response);
        let status = response.status.indicator; // "none", "minor", "major", "critical"

        if status != "none" {
            eprintln!("OpenAI API Status: {}", status);
            let io_error = std::io::Error::new(std::io::ErrorKind::Other, "OpenAI API is down");
            return Err(Box::new(io_error) as Box<dyn std::error::Error + Send + Sync>);
        } else {
            println!("OpenAI API is Operational");
            return Ok("OK".to_string());
        }
    } else if provider == "anthropic" {
        let status = "Anthropic API is Operational";
        println!("Anthropic API is Operational");
        
        return Ok(status.to_string());
    } else {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "Unknown provider");
        println!("Unknown provider");
        println!("{:#?}", io_error);
        return Err(Box::new(io_error) as Box<dyn std::error::Error + Send + Sync>);
    }
}

fn get_provider(model: &str) -> String {
    if ["gpt-3.5-turbo", "gpt-4"].contains(&model) {
        "openai".to_string()
    } else if ["claude-instant-1.2", "claude-2.1"].contains(&model) {
        "anthropic".to_string()
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
