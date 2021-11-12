mod game_elements;
mod board;
mod fen;

use board::Board;
use game_elements::{Position, Move};

fn main() {
    
    let board = Board::from_fen("2P5/1B4B1/8/3p3p/P3p3/1Q3B2/1p6/3P4 w - - 0 1").unwrap();
    println!("{}", board);

    let pos = Position::from_notation("b3").unwrap();
    let piece = board.get_pos(&pos).unwrap();
    piece.get_legal_moves(&pos, &board)
        .iter()
        .for_each(|m| {
            if let Move::NormalMove{from, to} = m {
                println!("{}", to.as_notation())
            } else if let Move::PawnPromotion{from, to, promote_to} = m {
                println!("{} -> {:?}", to.as_notation(), promote_to)
            }
        });


}
