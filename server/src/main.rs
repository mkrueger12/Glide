//#![deny(warnings)]
//#![allow(dead_code)]
#![forbid(unsafe_code)]
#[macro_use] extern crate rocket;
extern crate lazy_static;
use std::collections::HashMap;
use std::sync::Arc;

mod handlers;
mod providers;
mod config;
use handlers::model_router;
use config::settings;

use tokio::sync::{mpsc, RwLock};
use warp::reject::Reject;
use warp::ws::Message;
use warp::Filter;
use crate::config::settings::CONF;
use async_trait::async_trait;
use std::error::Error;
use std::borrow::Cow;

use rocket::State;
use rocket::tokio::sync::Mutex;
use rocket::serde::json::{Json, Value, json};
use rocket::serde::{Serialize, Deserialize};
mod payload_parse;

#[derive(Debug)]
pub struct MyError {
    pub message: String,
}

impl MyError {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn message(&self) -> &String {
        &self.message
    }
}

impl Reject for MyError {}

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


/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

// The type to represent the ID of a message.
type Id = usize;

// We're going to store all of the messages here. No need for a DB.
type MessageList = Mutex<Vec<String>>;
type Messages<'r> = &'r State<MessageList>;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]

struct Message<'r> {
    id: Option<Id>,
    message: Cow<'r, str>
}

#[post("/send")]
async fn main() {
    pretty_env_logger::init();

    // Keep track of all connected users, key is usize, value
    // is a websocket sender.
    let users = Users::default();

    // Turn our "state" into a new Filter...
    let users = warp::any().map(move || users.clone());
    let users_clone = users.clone();

    let provider_result = handlers::model_router::model_route(payload).await; // model router handles checking API status
            match provider_result {
                Ok(provider) => {
                    let response = handle_provider(provider, &users)
                        .await
                        .map_err(|e| warp::reject::custom(MyError::new(e.to_string())))?;
                    Ok::<_, warp::Rejection>(warp::reply::with_status(
                        response,
                        warp::http::StatusCode::OK,
                    ))
                }
                Err(e) => {
                    // Convert the error to a warp::Rejection
                    Err(warp::reject::custom(MyError::new(e.to_string())))
                }
            };



    let ip = CONF.as_ref().map(|settings| settings.generic.ip).unwrap();
    let port = CONF.as_ref().map(|settings| settings.generic.port).unwrap();

    let socket_addr: std::net::SocketAddr = (ip, port).into();

}


async fn user_message(
    my_id: usize,
    msg: Message,
    users: &Users,
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
            format!("<User#{}>: {:#?}", my_id, inner_string)
        }
        Err(err) => {
            // Handle the error case
            eprintln!("error: {:#?}", err);
            String::from("An error occurred")
        }
    };


    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in users.read().await.iter() {
        if my_id == uid {
            if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
    Ok(new_msg)
}

async fn handle_provider(
    provider: model_router::ProviderOptions,
    users: &Users,
) -> Result<String, Box<dyn std::error::Error>> {
    let (prompt, provider_name, model_name) = match provider {
        model_router::ProviderOptions::First(first_option) => {
            (first_option.prompt, first_option.provider, first_option.model)
        }
        model_router::ProviderOptions::Second(scnd_option) => {
            (scnd_option.prompt, scnd_option.provider, scnd_option.model)
        }
    };
    let msg = Message::text(&prompt);
    user_message(0, msg, users, &provider_name, &model_name).await
}
