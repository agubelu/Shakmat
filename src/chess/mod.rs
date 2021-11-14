mod board;
mod fen;
mod game_elements; 

pub use board::{Board, Piece};
pub use game_elements::{Color, Move, PieceType, CastlingRights, Position};

use game_elements::position;
use board::PieceArray;