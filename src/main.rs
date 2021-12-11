#[macro_use] extern crate rocket;

mod chess;
mod server;

use server::state::ServerState;
use server::handlers;
use std::sync::Mutex;

#[launch]
fn run() -> _ {
    rocket::build()
        .mount("/", handlers::get_routes())
        .manage(Mutex::from(ServerState::new()))
}