#[macro_use] extern crate rocket;

mod handlers;
mod state;
mod messages;

use state::ServerState;
use std::sync::Mutex;

#[launch]
fn run() -> _ {
    rocket::build()
        .mount("/", handlers::get_routes())
        .manage(Mutex::from(ServerState::new()))
}