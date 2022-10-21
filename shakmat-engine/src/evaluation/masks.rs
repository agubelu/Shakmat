use shakmat_core::BitBoard;

// A collection of useful bitboard masks for evaluation stuff
// They are generated automatically on startup using evaluation::init_evaluation()
pub static mut FILES: [BitBoard; 64] = [BitBoard::new(0); 64];
pub static mut RANKS: [BitBoard; 64] = [BitBoard::new(0); 64];
pub static mut WHITE_PASSED_PAWN: [BitBoard; 64] = [BitBoard::new(0); 64];
pub static mut BLACK_PASSED_PAWN: [BitBoard; 64] = [BitBoard::new(0); 64];
pub static mut KING_INNER_RING: [BitBoard; 64] = [BitBoard::new(0); 64];
pub static mut KING_OUTER_RING: [BitBoard; 64] = [BitBoard::new(0); 64];

// Some safe wrappers around the masks, since "static mut"s are inherently
// unsafe. The operations are totally safe however, since the masks are only
// modified during initialization, but the compiler can't prove this.
pub fn file(pos: u8) -> BitBoard {
    unsafe { FILES[pos as usize] }
}

pub fn white_passed_pawn(pos: u8) -> BitBoard {
    unsafe { WHITE_PASSED_PAWN[pos as usize] }
}

pub fn black_passed_pawn(pos: u8) -> BitBoard {
    unsafe { BLACK_PASSED_PAWN[pos as usize] }
}

pub fn king_inner_ring(pos: u8) -> BitBoard {
    unsafe { KING_INNER_RING[pos as usize] }
}

pub fn king_outer_ring(pos: u8) -> BitBoard {
    unsafe { KING_OUTER_RING[pos as usize] }
}