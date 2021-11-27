mod board;
mod fen;
mod game_elements; 
pub mod magic;

pub use board::{Board, Piece, BitBoard, BBBoard};
pub use game_elements::{Color, Move, PieceType, CastlingRights, Position};

use game_elements::position;
use board::PieceArray;