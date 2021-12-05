mod board;
mod fen;
mod game_elements; 
pub mod magic;

pub use board::{BitBoard, BBBoard};
pub use game_elements::{Color, PieceType, CastlingRights, BBMove, BBSquare};

use game_elements::position;