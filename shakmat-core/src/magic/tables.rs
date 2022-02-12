use crate::board::BitBoard;

pub static BLACK_PAWN_PUSHES: [BitBoard; 64] = include!("movetables/black_pawn_pushes.in");
pub static WHITE_PAWN_PUSHES: [BitBoard; 64] = include!("movetables/white_pawn_pushes.in");
pub static BLACK_PAWN_ATTACKS: [BitBoard; 64] = include!("movetables/black_pawn_attacks.in");
pub static WHITE_PAWN_ATTACKS: [BitBoard; 64] = include!("movetables/white_pawn_attacks.in");
pub static KING_MOVES: [BitBoard; 64] = include!("movetables/king_moves.in");
pub static KNIGHT_MOVES: [BitBoard; 64] = include!("movetables/knight_moves.in");
pub static BISHOP_MOVES: [BitBoard; 5248] = include!("movetables/bishop_moves.in");
pub static ROOK_MOVES: [BitBoard; 102400] = include!("movetables/rook_moves.in");
pub static EP_ATTACKS: [BitBoard; 64] = include!("movetables/enpassant_attacks.in");
