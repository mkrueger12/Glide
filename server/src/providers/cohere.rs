use crate::config::settings::CONF;
use dotenvy::dotenv;
use reqwest;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenCount {
    prompt_tokens: u32,
    response_tokens: u32,
    total_tokens: u32,
    billed_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiVersion {
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BilledUnits {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    api_version: ApiVersion,
    billed_units: BilledUnits,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CohereResponse {
    response_id: String,
    text: String,
    generation_id: String,
    token_count: TokenCount,
    meta: Meta,
}

// Function to interact with ChatGPT
pub async fn chat_with_cohere(
    input: &str,
    model: &str,
) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
    dotenv().expect("Error loading .env file");

    // Set your OpenAI API key
    let api_key = env::var("COHERE_KEY").expect("COHERE API KEY not set");

    print!("Running Cohere Chat");

    // Set up the HTTP client
    let client = reqwest::Client::new();

    let _ = model;

    eprint!("Request Payload: {}", &input);

    // Make the API request
    let cohere_endpoint: &String = CONF
        .as_ref()
        .map(|settings| &settings.cohere.endpoint)
        .unwrap();
    let res = client
        .post(cohere_endpoint)
        .header("accept", "application/json")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .body(input.to_string())
        .send()
        .await?;

    let body = res.text().await?;

    eprintln!("Cohere Response: {}", body);

    let response_result: Result<serde_json::Value, _> = serde_json::from_str(&body);

    match response_result {
        Ok(response) => Ok(response),
        Err(e) => Err(Box::new(e)),
    }
}
