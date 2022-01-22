mod evaluation;
mod minimax;

use shakmat_core::{Board, Move};

pub fn find_best_move(board: &Board) -> Option<Move> {

    let best = minimax::find_best(board, 6);
    println!("Evaluation: {}", best.evaluation());
    best.get_move()
}