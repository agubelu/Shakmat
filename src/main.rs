mod chess;
mod server;

use server::state::ServerState;
use server::handlers;

use std::{io::{Error, Result}, sync::Mutex};
use rocket::State;

use chess::{Board, Position, Move, Color};


#[macro_use] extern crate rocket;


#[launch]
fn run() -> _ {
    rocket::build()
        .mount("/", routes![handlers::create_default_game,
                        handlers::create_game_from_fen])
        .manage(Mutex::from(ServerState::new()))
}



/*
fn main() {
    
    let board = Board::default();
    (1..7).into_iter().for_each(|i| println!("{} -> {}", i, find_all_moves_depth(&board, 0, i)));
    
}



fn find_all_moves_depth(board: &Board, cur_depth: u32, max_depth: u32) -> u64 {
    if cur_depth == max_depth {
        return 1;
    }

    let mut res = 0;
    for mv in board.get_current_turn_moves() {
        let new_board = board.make_move(mv, false).unwrap();
        res += find_all_moves_depth(&new_board, cur_depth + 1, max_depth);
    }

    return res;
}
*/  