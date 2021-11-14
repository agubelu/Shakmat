mod chess;

use chess::{Board, Position, Move, Color};

fn main() {
    
    let mut board = Board::from_fen("1k/8/8/4Pp2/4N1pP/8/8/R3K3 b Q - 0 1").unwrap();
    println!("{}", board);

    board.get_current_turn_moves().iter().for_each(|m| println!("{}", m));

    let e5 = Position::from_notation("g4").unwrap();
    let f6 = Position::from_notation("h3").unwrap();

    board = board.make_move(Move::NormalMove{from: e5, to: f6}, true).unwrap();

    println!("{}", board);
    
    
}
