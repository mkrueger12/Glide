#[macro_use]
extern crate rocket;

mod config;
mod handlers;
mod payload_parse;
mod providers;

#[launch]
fn rocket() -> _ {
    rocket::build().attach(payload_parse::stage())
}
