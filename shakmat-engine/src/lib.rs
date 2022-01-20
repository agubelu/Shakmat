mod evaluation;
mod minimax;

use shakmat_core::{Board, Move};

pub fn find_best_move(board: &Board) -> Option<Move> {
    minimax::find_best(board, 0, 4).get_move()
}