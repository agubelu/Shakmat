//#[macro_use] extern crate rocket;

mod chess;
//mod server;

//use server::state::ServerState;
//use server::handlers;

//use std::fs::File;
//use std::io::Write;
//use std::{ops::BitAnd, sync::Mutex};
//use std::time::Instant;

use chess::{Board, Move, Position, BitBoard, BBBoard, BBSquare};
use chess::magic;

fn main() {
    let board = BBBoard::from_fen("rnbqk2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
    println!("{}", board);
    board.pseudolegal_moves(!board.turn_color()).iter().for_each(|m| println!("{}", m));
}
/*
#[launch]
fn run() -> _ {
    rocket::build()
        .mount("/", handlers::get_routes())
        .manage(Mutex::from(ServerState::new()))
}
*/