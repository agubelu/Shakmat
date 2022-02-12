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
            Knight => 2,
            Bishop => 4,
            Rook => 6,
            Queen => 8,
            King => 10,
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