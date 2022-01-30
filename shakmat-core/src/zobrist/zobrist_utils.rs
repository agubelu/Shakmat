use crate::{PieceType, Color};
use crate::game_elements::CastlingRights;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

const N_KEYS: usize = 793;
/*
 The 781 elements in the array are logically distributed as follows:
 - 64 * 6 for all possible squares of all types of white pieces (0-383)
 - 64 * 6 for all possible squares of all types of black pieces (384-767)
 - 16 for all possible castling options (WK, WQ, BK, BQ) (768-784)
 - 8 for the files of the current e.p. square (784-791)
 - 1 to signal that Black is to move (792)
*/
pub static mut ZOBRIST_VALUES: [u64; N_KEYS] = [0; N_KEYS];

pub fn init_zobrist_keys() {
    // Fix the seed so the keys are consistent between executions
    let mut rng = StdRng::seed_from_u64(1337);
    
    // Modifying a static region of memory is unsafe since it may
    // cause data races. In our case, this is done when the program
    // starts up, even before the web server starts running,
    // so it should cause no problems
    unsafe {
        ZOBRIST_VALUES.iter_mut().for_each(|x| *x = rng.gen());
    }
}

// Subsequent reads are also unsafe, since Rust cannot prove that there are no
// data races, however we do not modify the array after initialization so
// no data races can occur
pub fn get_key_for_piece(piece: PieceType, color: Color, square: u8) -> u64 {
    unsafe {
        ZOBRIST_VALUES[color.to_index() * 384 + piece.to_index() * 64 + square as usize]
    }
}

pub fn get_key_castling(cr: &CastlingRights) -> u64 {
    unsafe {
        ZOBRIST_VALUES[768 + cr.to_index()]
    }
}

pub fn get_key_ep_square(square: u8) -> u64 {
    unsafe {
        ZOBRIST_VALUES[784 + (square as usize % 8)]
    }
}

pub fn get_key_black_turn() -> u64 {
    unsafe {
        ZOBRIST_VALUES[N_KEYS - 1]
    }
}