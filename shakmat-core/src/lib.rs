mod board;
mod fen;
mod game_elements; 
pub mod magic;
mod zobrist;

pub use board::{Board, BitBoard, Pieces};
pub use fen::DEFAULT_FEN;
pub use game_elements::{Move, Color, PieceType, Square};
pub use magic as move_gen;