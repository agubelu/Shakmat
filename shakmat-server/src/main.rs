#[macro_use] extern crate rocket;
use rocket::config::Config;

mod handlers;
mod state;
mod messages;

use state::ServerState;
use std::env::args;
use std::sync::Mutex;
use shakmat_engine::ShakmatEngine;

const DEFAULT_PORT: u16 = 8000;

#[launch]
fn run() -> _ {
    // Allow the first CLI arg to set the port if it's a valid number
    let port = args().nth(1).map(|s| s.parse().unwrap_or(DEFAULT_PORT)).unwrap_or(DEFAULT_PORT);
    let config = Config {port, ..Config::default()};

    // Init stuff in the engine
    shakmat_engine::init_evaluation();

    rocket::build()
        .configure(config)
        .mount("/", handlers::get_routes())
        .manage(Mutex::from(ServerState::new()))
        .manage(ShakmatEngine::default())
}