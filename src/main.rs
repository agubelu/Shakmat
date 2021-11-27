#[macro_use] extern crate rocket;

mod chess;
mod server;

use server::state::ServerState;
use server::handlers;

use std::fs::File;
use std::io::Write;
use std::{ops::BitAnd, sync::Mutex};
use std::time::Instant;

use chess::{Board, Move, Position, BitBoard, BBBoard};
use chess::magic;

fn main() {
    println!("Board -> {}", std::mem::size_of::<Board>());
    println!("BBBoard -> {}", std::mem::size_of::<BBBoard>());
}
/*
#[launch]
fn run() -> _ {
    rocket::build()
        .mount("/", handlers::get_routes())
        .manage(Mutex::from(ServerState::new()))
}
*/