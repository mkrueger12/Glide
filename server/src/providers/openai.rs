#![forbid(unsafe_code)]

use crate::config::settings::CONF;
use dotenvy::dotenv;
use reqwest;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct ChatGptRequest {
    prompt: String,
    max_tokens: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatGptResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    //system_fingerprint: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GptResponse {
    data: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    index: u32,
    message: Message,
    finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

// Function to interact with ChatGPT
pub async fn chat_with_gpt(
    input: &str,
    model: &str,
) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
    println!("input: {}", &input);

    dotenv().expect("Error loading .env file");

    // Set your OpenAI API key
    let api_key = env::var("OPENAI_KEY").expect("OPENAI_KEY not set");

    print!("Running OpenAI Chat");

    // Set up the HTTP client
    let client = reqwest::Client::new();

    // Set up the request payload
    let request_payload = format!(
        r#"{{ 
        "model": "{}",
        "messages": [
          {{
            "role": "system",
            "content": "You are a helpful assistant."
          }},
          {{
            "role": "user",
            "content": "{}"
          }}
        ]
      }}"#,
        model, input
    );

    eprint!("Request Payload: {}", request_payload);

    // Make the API request
    let openai_endpoint: &String = CONF
        .as_ref()
        .map(|settings| &settings.openai.endpoint)
        .unwrap(); //This unwrap will cause a panic if empty
    let res = client
        .post(openai_endpoint)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .body(request_payload)
        .send()
        .await?;

    let body = res.text().await?;

    eprintln!("OpenAI Response: {}", body);

    let response_result: Result<serde_json::Value, _> = serde_json::from_str(&body);

    match response_result {
        Ok(response) => Ok(response),
        Err(e) => Err(Box::new(e)),
    }
}
