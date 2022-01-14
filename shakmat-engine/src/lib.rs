use rand::prelude::*;
use shakmat_core::{Board, Move};

pub fn find_best_move(board: &Board) -> Option<Move> {
    let mut rng = thread_rng();
    let moves = board.legal_moves();
    moves.choose(&mut rng).copied()
}