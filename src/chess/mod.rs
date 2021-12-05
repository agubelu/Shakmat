mod board;
mod fen;
mod game_elements; 
pub mod magic;

pub use board::{BitBoard, Board};
pub use game_elements::{Color, PieceType, CastlingRights, Move, Square};

use game_elements::position;