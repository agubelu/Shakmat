mod chess;

use chess::{Board, Position, Move, Color};

use std::io::{stdin,stdout,Write};

fn main() {
    
    let board = Board::from_fen("k2r1r2/8/8/8/8/8/8/R3K2R w KQk - 0 1").unwrap();
    println!("{}", board);

    let pos = Position::from_notation("e1").unwrap();
    board.get_pos(&pos).unwrap().get_legal_moves(&pos, &board)
        .iter().for_each(|m| println!("{}", m));


}
