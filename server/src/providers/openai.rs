use reqwest::{self, Error, Response};
use serde::{Deserialize, Serialize};
use std::env;
use std::io;

#[derive(Debug, Serialize, Deserialize)]
struct ChatGptRequest {
    prompt: String,
    max_tokens: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatGptResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    text: String,
}


pub fn main() {
    // Example usage
    let input_text = "Translate this English text to French: ";
    match chat_with_gpt(input_text) {
        Ok(response) => println!("ChatGPT Response: {}", response),
        Err(err) => eprintln!("Error: {:?}", err),
    }
}


// Function to interact with ChatGPT
pub fn chat_with_gpt(input: &str) -> Result<String, reqwest::Error> {
    // Set up the request payload
    let request_payload = serde_json::json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant." // this should be provided by user or YAML
            },
            {
                "role": "user",
                "content": &input // Update to user input
            }
        ]
    });

    // Set your OpenAI API key
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

    // Set up the HTTP client
    let client = reqwest::blocking::Client::new();
    

    // Make the API request
    let response: ChatGptResponse = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_payload)
        .send()?
        .json()?;

    // Extract and return the response text
    let choice = response.choices.get(0).ok_or_else(|| {
        reqwest::Error::custom(io::ErrorKind::Other, "No response choices found in the API response")
    })?;

    Ok(choice.text.clone())
}


