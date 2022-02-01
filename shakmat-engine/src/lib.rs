mod evaluation;
mod search;
mod trasposition;

use shakmat_core::{Board, Move};

pub use trasposition::TTData;

pub fn find_best_move(board: &Board) -> Option<Move> {

    let best = search::find_best(board, 6);
    println!("Evaluation: {}", best.evaluation());
    best.get_move()
}