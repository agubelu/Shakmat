//#[macro_use] extern crate rocket;

mod chess;
//mod server;

//use server::state::ServerState;
//use server::handlers;

//use std::fs::File;
//use std::io::Write;
//use std::{ops::BitAnd, sync::Mutex};
use std::time::Instant;

use chess::{BitBoard, Board};

fn main() {
    
    let board = Board::default();
    let t = Instant::now();
    println!("{}", board.perft(7));
    let time = t.elapsed().as_millis() as f64 / 1000.0;
    println!("{:.2}", time);
    
}

/*
#[launch]
fn run() -> _ {
    rocket::build()
        .mount("/", handlers::get_routes())
        .manage(Mutex::from(ServerState::new()))
}
*/