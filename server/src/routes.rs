#![deny(warnings)]

use warp::Filter;
mod providers;

#[tokio::main]
pub async fn routes() {
    pretty_env_logger::init();

    // We'll start simple, and gradually show how you combine these powers
    // into super powers!

    // GET /
    let hello_world = warp::path::end().map(|| "Hello, World at root!");

    // GET /openai
    let hi = warp::path("openai").map(|| providers::openai::chat_with_gpt("Hello!").await.unwrap());

    // How about multiple segments? First, we could use the `path!` macro:
    //
    // GET /hello/from/warp
    let hello_from_warp = warp::path!("hello" / "from" / "warp").map(|| "Hello from warp!");

    // Fine, but how do I handle parameters in paths?
    //
    // GET /sum/:u32/:u32
    let sum = warp::path!("sum" / u32 / u32).map(|a, b| format!("{} + {} = {}", a, b, a + b));

}