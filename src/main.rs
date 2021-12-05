//#[macro_use] extern crate rocket;

mod chess;
//mod server;

//use server::state::ServerState;
//use server::handlers;

//use std::fs::File;
//use std::io::Write;
//use std::{ops::BitAnd, sync::Mutex};
//use std::time::Instant;

use chess::{BitBoard, Board};

fn main() {
    
    //let board = BBBoard::default();
    let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/P3P2P/R3K2R w KQkq - 0 1").unwrap();
    
    for mv in board.pseudolegal_moves(chess::Color::White) {
        println!("{}", mv);
        println!("{}", board.make_move(mv, false).unwrap());
    }
}

/*
#[launch]
fn run() -> _ {
    rocket::build()
        .mount("/", handlers::get_routes())
        .manage(Mutex::from(ServerState::new()))
}
*/