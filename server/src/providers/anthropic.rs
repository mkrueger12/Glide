use reqwest;
use serde::{Deserialize, Serialize};
use std::env;
use dotenvy::dotenv;
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
pub async fn chat_with_gpt(input: &str) -> Result<String, Box<dyn Error + Send + Sync>> {

    println!("input: {}", &input);

    dotenv().ok();

    // Set your OpenAI API key
    let api_key = env::var("OPENAI_KEY").expect("OPENAI_KEY not set");

    // Set up the HTTP client
    let client = reqwest::Client::new();
        
    // Set up the request payload
    let request_payload = format!(r#"{{ 
        "model": "gpt-3.5-turbo",
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
      }}"#, input);

    // Make the API request
    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .body(request_payload)
        .send()
        .await?;

        let body = res.text().await?;

        eprintln!("OpenAI Response: {}", body);

        let response_result: Result<ChatGptResponse, _> = serde_json::from_str(&body);

        let response = match response_result {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Failed to parse response: {}", err);
                let io_error = std::io::Error::new(std::io::ErrorKind::Other, "Failed to parse JSON");
                return Err(Box::new(io_error) as Box<dyn std::error::Error + Send + Sync>);
            }
        };

    // Extract and return the response text
    let choice = response.choices.get(0).unwrap();
    let text = choice.message.content.clone();

    //Ok(response.data[0].message.content.clone())

    Ok(text)
}

