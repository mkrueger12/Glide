#![deny(warnings)]

use warp::Filter;
mod providers;



#[derive(Deserialize, Serialize)]
struct Payload {
    model: String,
    message: String,
}


#[tokio::main]
pub async fn routes() {
    pretty_env_logger::init();

    // We'll start simple, and gradually show how you combine these powers
    // into super powers!

    // GET /
    //let hello_world = warp::path::end().map(|| "Hello, World at root!");

    // GET /api/ws -> websocket upgrade
    let sockets = warp::path("api/ws")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(users)
        .map(|ws: warp::ws::Ws, users| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket: WebSocket| user_connected(socket, users))
        });

    // GET / -> index html
    let index = warp::path::end().map(|| warp::reply::html(INDEX_HTML));

    /// POST /api/v1 with JSON body
    // POST /employees/:rate  {"name":"Sean","rate":2}
    let client_payload = warp::post()
        .and(warp::path("api" / "v1"))
        .and(warp::path::param::String())
        // Only accept bodies smaller than 16kb...
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .map(|message, mut model: Payload| {
            model.message = message;
            warp::reply::json(&employee)
        });

    let routes = index.or(sockets).or(client_payload);

    routes

}