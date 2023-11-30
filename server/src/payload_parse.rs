
use crate::handlers;
use handlers::model_router;

use crate::providers;

use rocket::State;
use rocket::tokio::sync::Mutex;
use rocket::serde::json::{Json, Value, json};
use rocket::serde::{Serialize, Deserialize};


#[async_trait]
trait Provider: Send + Sync {
    async fn chat(&self, msg: &str, model_name: &str) -> Result<String, Box<dyn Error + Send + Sync>>;
}

struct OpenAI;
struct Cohere;

#[async_trait]
impl Provider for OpenAI {
    async fn chat(&self, msg: &str, model_name: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        providers::openai::chat_with_gpt(msg, model_name).await
    }
}

#[async_trait]
impl Provider for Cohere {
    async fn chat(&self, msg: &str, model_name: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        providers::cohere::chat_with_cohere(msg, model_name).await
    }
}


#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Payload {
    // This comes from the client
    pub model: Vec<String>,
    pub prompt: Vec<String>,
}

#[post("/post", format = "json", data = "<payload>")]
async fn new(payload: Payload) -> Result<Json<Value>, MyError> {

    let provider_result = handlers::model_router::model_route(payload).await;

    let (prompt, provider_name, model_name) =  handle_provider(provider_result.unwrap()).await;

    let new_msg = user_message(prompt, provider_name, model_name).await;


}


async fn handle_provider( provider: model_router::ProviderOptions) -> Result<String, Box<dyn std::error::Error>> {
    let (prompt, provider_name, model_name) = match provider {
        model_router::ProviderOptions::First(first_option) => {
            (first_option.prompt, first_option.provider, first_option.model)
        }
        model_router::ProviderOptions::Second(scnd_option) => {
            (scnd_option.prompt, scnd_option.provider, scnd_option.model)
        }
    };
    (prompt, provider_name, model_name)
}

async fn user_message(
    msg: String,
    provider: &str,
    model_name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        eprintln!("new user msg: {}", s);
        s
    } else {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Non-text message",
        )));
    };
    // add comment
    print!("provider: {}", provider);
    print!("model_name: {}", model_name);

    let provider: Box<dyn Provider> = match provider {
        "openai" => Box::new(OpenAI),
        "cohere" => Box::new(Cohere),
        _ => return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Invalid Provider",
        ))),
    };

    let model_response = provider.chat(msg, model_name).await;

    // Extract the inner string using pattern matching
    let new_msg = match model_response {
        Ok(inner_string) => {
            // Do something with the inner string
            format!("{:#?}", inner_string)
        }
        Err(err) => {
            // Handle the error case
            eprintln!("error: {:#?}", err);
            String::from("An error occurred")
        }
    };

    Ok(new_msg)
}


#[catch(404)]
fn not_found() -> Value {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

pub fn stage() -> rocket::fairing::AdHoc {
    rocket::fairing::AdHoc::on_ignite("JSON", |rocket| async {
        rocket.mount("/api/vi", routes![new])
            .register("/api/v1", catchers![not_found])
            .manage(MessageList::new(vec![]))
    })
}

