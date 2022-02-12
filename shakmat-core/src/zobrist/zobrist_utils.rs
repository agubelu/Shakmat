use crate::{PieceType, Color};
use crate::game_elements::CastlingRights;

/*
 The 793 elements in the array are logically distributed as follows:
 - 768 for all possible squares of all types of pieces (0-767)
    - The position of the array for any given piece is: 64 * kind_of_piece + 8*row + file
    - kind_of_piece is: {black/white} pawn, knight, bishop, rook, queen, king
 - 16 for all possible castling options (WK, WQ, BK, BQ) (768-783)
 - 8 for the files of the current e.p. square (784-791)
 - 1 to signal that White is to move (792)
*/
pub static ZOBRIST_VALUES: [u64; 793] = include!("rng_values.in");


pub fn get_key_for_piece(piece: PieceType, color: Color, square: u8) -> u64 {
    ZOBRIST_VALUES[64 * (piece.to_index() + color.to_index()) + square as usize]
}

pub fn get_key_castling(cr: &CastlingRights) -> u64 {
    ZOBRIST_VALUES[768 + cr.index()]
}

pub fn get_key_ep_square(square: u8) -> u64 {
    ZOBRIST_VALUES[784 + (square as usize % 8)]
}

pub fn get_key_white_turn() -> u64 {
    ZOBRIST_VALUES[792]
}