mod evaluation;
mod minimax;

use shakmat_core::{Board, Move};

pub fn find_best_move(board: &Board) -> Option<Move> {

    let best = minimax::find_best(board, 5);
    println!("Evaluation: {}", best.score());
    best.get_move()
}