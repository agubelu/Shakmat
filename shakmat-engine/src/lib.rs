mod evaluation;
mod search;
mod trasposition;

use shakmat_core::{Board, Move};

// Exports
pub use search::is_draw_by_repetition;


pub fn find_best_move(board: &Board, past_positions: &[u64]) -> Option<Move> {
    let best = search::find_best(board, 6, past_positions);
    println!("Evaluation: {}", best.eval);
    best.best
}