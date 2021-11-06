mod game_elements;
mod board;
mod fen;

use board::Board;
use game_elements::{Position, Move};

fn main() {
    let board = Board::from_fen("r4r2/PPpqppQk/2n4p/3p1N2/3P4/2P1P3/pP3PPP/r1n2RK1 b - - 0 14").unwrap();
    println!("{}", board);

    let pos = Position::from_notation("c3").unwrap();
    let piece = board.get_pos(&pos).unwrap();
    piece.get_legal_moves(&pos, &board)
        .iter()
        .for_each(|m| {
            if let Move::NormalMove{from, to} = m {
                println!("{}", to.as_notation())
            } else if let Move::PawnPromotion{from, to, promote_to} = m {
                println!("{} -> {:?}", to.as_notation(), promote_to)
            }
        })
}
