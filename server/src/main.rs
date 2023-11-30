#[macro_use] extern crate rocket;

mod payload_parse;
mod handlers;
mod providers;
mod config;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(payload_parse::stage())
}