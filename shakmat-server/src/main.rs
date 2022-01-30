#[macro_use] extern crate rocket;

mod handlers;
mod state;
mod messages;

use state::ServerState;
use std::sync::Mutex;

#[launch]
fn run() -> _ {
    // Initialize the random values for the zobrist keys that
    // the board uses before launching the server
    shakmat_core::init_zobrist_keys();

    rocket::build()
        .mount("/", handlers::get_routes())
        .manage(Mutex::from(ServerState::new()))
}