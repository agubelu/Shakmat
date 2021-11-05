mod game_elements;
mod board;
mod fen;

use board::Board;

fn main() {
    let board = Board::from_fen("r4r2/pppqppQk/2n4p/3p1N2/3P4/2P1P3/PP3PPP/RN3RK1 b - - 0 14").unwrap();
    //let board = Board::default();
    println!("{}", board);
}
