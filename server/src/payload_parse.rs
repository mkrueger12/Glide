use crate::handlers::model_router;

use crate::providers;

use std::error::Error;

use crate::handlers::model_router::Payload;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::serde::json::{json, Value};
use rocket::serde::Serialize;

use rocket::http::Status;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ModelResponse {
    response: Result<serde_json::Value, String>,
}

#[async_trait]
trait Provider: Send + Sync {
    async fn chat(
        &self,
        msg: &str,
        model_name: &str,
    ) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>>;
}

struct OpenAI;
struct Cohere;

#[async_trait]
impl Provider for OpenAI {
    async fn chat(
        &self,
        msg: &str,
        model_name: &str,
    ) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
        providers::openai::chat_with_gpt(msg, model_name).await
    }
}

#[async_trait]
impl Provider for Cohere {
    async fn chat(
        &self,
        msg: &str,
        model_name: &str,
    ) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
        providers::cohere::chat_with_cohere(msg, model_name).await
    }
}

#[post("/post", format = "json", data = "<payload>")]
async fn new(
    payload: Json<Payload>,
) -> Result<status::Accepted<Json<Value>>, status::Custom<String>> {
    let provider_result = model_router::model_route(payload.into_inner())
        .await
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;

    let (prompt, provider_name, model_name) = handle_provider(provider_result)
        .await
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;

    let new_msg = user_message(prompt, &provider_name, &model_name)
        .await
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;

    Ok(status::Accepted(Json(new_msg)))
}

async fn handle_provider(
    provider: model_router::ProviderOptions,
) -> Result<(String, String, String), Box<dyn std::error::Error>> {
    let (prompt, provider_name, model_name) = match provider {
        model_router::ProviderOptions::First(first_option) => (
            first_option.prompt,
            first_option.provider,
            first_option.model,
        ),
        model_router::ProviderOptions::Second(scnd_option) => {
            (scnd_option.prompt, scnd_option.provider, scnd_option.model)
        }
    };
    Ok((prompt, provider_name, model_name))
}

async fn user_message(
    msg: String,
    provider: &str,
    model_name: &str,
) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
    // Skip any non-Text messages...
    if msg.is_empty() {
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
        _ => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid Provider",
            )))
        }
    };

    Ok(provider.chat(&msg, model_name).await?)
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
        rocket
            .mount("/api/v1", routes![new])
            .register("/api/v1", catchers![not_found])
    })
}
