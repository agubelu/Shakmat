mod chess;

use chess::{Board, Position, Move, Color};

use std::io::{stdin,stdout,Write};

fn main() {
    
    let board = Board::from_fen("R7/8/r7/2N3K1/8/1P6/P1P5/7B w - - 0 1").unwrap();
    println!("{}", board);

    loop {        
        let mut s =String::new();
        print!("Pos: ");
        stdout().flush();
        stdin().read_line(&mut s).expect("Did not enter a correct string");

        match Position::from_notation(s.trim()) {
            Ok(position) => {
                println!("White: {}, black: {}", position.is_attacked_by(&board, Color::White),
            position.is_attacked_by(&board, Color::Black));
            },
            Err(e) => println!("{}", e)
        }
    }


}
