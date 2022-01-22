use shakmat_core::{Board, Color::*, Color};

// Evaluate how favorable a position is for Black or White
// Positive scores favor White, while negative scores favor Black
// The maximum or minimum possible score denotes a checkmate
pub fn evaluate_position(board: &Board) -> i32 {
    piece_score(board, White) - piece_score(board, Black)
    + control_score(board, White) - control_score(board, Black)
}

// Computes the total piece score of a color, where:
// - Pawns: 100 point
// - Knights and Bishops: 300 points
// - Rooks: 500 points
// - Queen: 900 points
fn piece_score(board: &Board, color: Color) -> i32 {
    let pieces = board.get_pieces(color);
    
    let score = 100 * pieces.pawns.count() +
    300 * pieces.knights.count() +
    300 * pieces.bishops.count() +
    500 * pieces.rooks.count() +
    900 * pieces.queens.count();

    score as i32
}

fn control_score(board: &Board, color: Color) -> i32 {
    board.get_attack_bitboard(color).count() as i32 * 5
}