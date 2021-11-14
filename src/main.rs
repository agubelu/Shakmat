mod chess;

use chess::{Board, Position, Move, Color};

use std::io::{stdin,stdout,Write};

fn main() {
    
    let board = Board::default();
    println!("{}", board);

    let moves = board.get_current_turn_moves();
    println!("{} moves:", moves.len());
    moves.iter().for_each(|m| println!("{}", m));

    println!("Size of Board: {} bytes", std::mem::size_of::<Board>())
}
