use crate::game_elements::{Color, Color::*};
use PieceType::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    pub const fn to_index(&self) -> usize {
        match self {
            Pawn => 0,
            Knight => 1,
            Bishop => 2,
            Rook => 3,
            Queen => 4,
            King => 5,
        }
    }
    pub fn as_char(&self, color: Color) -> char {
        match (color, self) {
            (White, Pawn) => '♙',
            (White, Knight) => '♘',
            (White, Bishop) => '♗',
            (White, Rook) => '♖',
            (White, Queen) => '♕',
            (White, King) => '♔',
            (Black, Pawn) => '♟',
            (Black, Knight) => '♞',
            (Black, Bishop) => '♝',
            (Black, Rook) => '♜',
            (Black, Queen) => '♛',
            (Black, King) => '♚',
        }
    }
}