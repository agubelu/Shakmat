#[macro_use] extern crate rocket;

use rocket::{Request, Response};
use rocket::http::Header;
use rocket::config::Config;
use rocket::fairing::{Fairing, Info, Kind};

mod handlers;
mod state;
mod messages;

use state::ServerState;
use std::env::args;
use std::sync::Mutex;
use shakmat_engine::ShakmatEngine;

const DEFAULT_PORT: u16 = 8000;

// Aux struct to allow requests from any origin, in order to be
// able to use browser-based front-ends
pub struct CORS;

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
        .manage(Mutex::from(ShakmatEngine::default()))
        .attach(CORS)
}

 
#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}