use crate::chess::{BitBoard, Color};
use super::masks::{BISHOP_BLOCKERS_MASK, ROOK_BLOCKERS_MASK};
use super::magics::{BISHOP_MAGICS, BISHOP_OFFSETS, BISHOP_SHIFTS, ROOK_MAGICS, ROOK_OFFSETS, ROOK_SHIFTS};
use super::tables::{BISHOP_MOVES, ROOK_MOVES, KING_MOVES, KNIGHT_MOVES, BLACK_PAWN_ATTACKS,
                    WHITE_PAWN_ATTACKS, BLACK_PAWN_PUSHES, WHITE_PAWN_PUSHES};

// The movement of the pawn for captures and for pushes is different
// so we provide different functions since they must be filtered differently
pub fn pawn_attacks(pos: usize, color: Color) -> BitBoard {
    match color {
        Color::White => WHITE_PAWN_ATTACKS[pos],
        Color::Black => BLACK_PAWN_ATTACKS[pos]
    }
}

pub fn pawn_pushes(pos: usize, color: Color) -> BitBoard {
    match color {
        Color::White => WHITE_PAWN_PUSHES[pos],
        Color::Black => BLACK_PAWN_PUSHES[pos]
    }
}

pub fn bishop_moves(pos: usize, blockers: BitBoard) -> BitBoard {
    let i = (blockers & BISHOP_BLOCKERS_MASK[pos])
                .wrapping_mul(BISHOP_MAGICS[pos]) >> BISHOP_SHIFTS[pos];
    BISHOP_MOVES[BISHOP_OFFSETS[pos] + i as usize]
}

pub fn rook_moves(pos: usize, blockers: BitBoard) -> BitBoard {
    let i = (blockers & ROOK_BLOCKERS_MASK[pos])
                .wrapping_mul(ROOK_MAGICS[pos]) >> ROOK_SHIFTS[pos];
    ROOK_MOVES[ROOK_OFFSETS[pos] + i as usize]
}   

pub fn queen_moves(pos: usize, blockers: BitBoard) -> BitBoard {
    bishop_moves(pos, blockers) | rook_moves(pos, blockers)
}

pub fn king_moves(pos: usize) -> BitBoard {
    KING_MOVES[pos]
}

pub fn knight_moves(pos: usize) -> BitBoard {
    KNIGHT_MOVES[pos]
}