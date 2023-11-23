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
pub async fn chat_with_cohere(input: &str, model: &str) -> Result<String, Box<dyn Error + Send + Sync>> {

    println!("input: {}", &input);

    dotenv().ok();

    // Set your OpenAI API key
    let api_key = env::var("OPENAI_KEY").expect("OPENAI_KEY not set");

    // Set up the HTTP client
    let client = reqwest::Client::new();
        
    // Set up the request payload
    let request_payload = format!(r#"{{
        "model": {},
        "message": {},
      }}"#, model, input);

      r#"curl 
      --request POST \
      --url https://api.cohere.ai/v1/chat \
      --header 'accept: application/json' \
      --header 'content-type: application/json' \
      --header 'Authorization: Bearer <<apiKey>>'
      --data '
      {
        "chat_history": [
          {"role": "USER", "message": "Who discovered gravity?"},
          {"role": "CHATBOT", "message": "The man who is widely credited with discovering gravity is Sir Isaac Newton"}
        ],
        "message": "What year was he born?",
        "model":"command-light"
        "connectors": [{"id": "web-search"}]
      }'"#;

    // Make the API request
    let res = client
        .post("https://api.anthropic.com/v1/complete")
        .header("accept", "application/json")
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .header("x-api-key", api_key)
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


