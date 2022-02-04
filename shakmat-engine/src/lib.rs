mod evaluation;
mod search;
mod trasposition;

use shakmat_core::{Board, Move};

pub use trasposition::TTData;

pub fn find_best_move(board: &Board, past_positions: &[u64]) -> Option<Move> {

    let best = search::find_best(board, 6, past_positions);
    println!("Evaluation: {}", best.eval);
    best.best
}