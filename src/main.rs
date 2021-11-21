#[macro_use] extern crate rocket;

mod chess;
mod server;

use server::state::ServerState;
use server::handlers;

use std::sync::Mutex;

use chess::{Board, Move};
/*
fn main() {
    let board = Board::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10").unwrap();
    (1..6).into_iter().for_each(|i| println!("{} -> {}", i, perft(&board, 0, i)));
}

fn perft(board: &Board, cur_depth: u64, max_depth: u64) -> u64 {
    if cur_depth == max_depth {
        return 1;
    }

    let mut moves = 0;
    for mv in board.get_current_turn_moves() {
        let new_board = board.make_move(mv, false).unwrap();
        moves += perft(&new_board, cur_depth + 1, max_depth);
    }

    return moves;
}
*/

#[launch]
fn run() -> _ {
    rocket::build()
        .mount("/", handlers::get_routes())
        .manage(Mutex::from(ServerState::new()))
}
