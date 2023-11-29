//#![deny(warnings)]
//#![allow(dead_code)]
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


/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // Keep track of all connected users, key is usize, value
    // is a websocket sender.
    let users = Users::default();

    // Turn our "state" into a new Filter...
    let users = warp::any().map(move || users.clone());
    let users_clone = users.clone();

    // POST /api/v1 with JSON body {"model":["gpt-3.5-turbo"],"message": ["hello"]}
    let client_payload = warp::post()
        .and(warp::path!("api" / "v1"))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<model_router::Payload>())
        .and(users_clone)
        .and_then(|payload: model_router::Payload, users: Users| async move {
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
            }
        })
        .recover(|err: warp::Rejection| async move {
            if err.is_not_found() {
                return Ok(warp::reply::with_status(
                    "Not Found",
                    warp::http::StatusCode::NOT_FOUND,
                ));
            }
            Err(err)
        });


    let ip = CONF.as_ref().map(|settings| settings.generic.ip).unwrap();
    let port = CONF.as_ref().map(|settings| settings.generic.port).unwrap();

    let socket_addr: std::net::SocketAddr = (ip, port).into();

    warp::serve(client_payload).run(socket_addr).await;
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

    print!("provider: {}", provider);
    print!("model_name: {}", model_name);

    let model_response = if provider == "openai" {
        providers::openai::chat_with_gpt(msg, model_name).await
    } else if provider == "cohere" {
        providers::cohere::chat_with_cohere(msg, model_name).await
    } else {
        println!("Invalid provider");
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Invalid Provider",
        )));
    };

    //let model_response = providers::openai::chat_with_gpt(msg).await;

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

    //let new_msg = format!("<User#{}>: {:#?}", my_id, msg);

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
