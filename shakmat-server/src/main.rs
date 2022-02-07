#[macro_use] extern crate rocket;
use rocket::config::Config;

mod handlers;
mod state;
mod messages;

use state::ServerState;
use std::env::args;
use std::sync::Mutex;

const DEFAULT_PORT: u16 = 8000;

#[launch]
fn run() -> _ {
    // Initialize the random values for the zobrist keys that
    // the board uses before launching the server
    shakmat_core::init_zobrist_keys();

    // Allow the first CLI arg to set the port if it's a valid number
    let port = args().nth(1).map(|s| s.parse().unwrap_or(DEFAULT_PORT)).unwrap_or(DEFAULT_PORT);
    let config = Config {port, ..Config::default()};

    rocket::build()
        .configure(config)
        .mount("/", handlers::get_routes())
        .manage(Mutex::from(ServerState::new()))
}