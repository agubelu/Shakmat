use crate::game_elements::{Color, Color::*};
use PieceType::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    // For arrays
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

    // For zobrist keys
    pub const fn to_zobrist_index(&self) -> usize {
        match self {
            Pawn => 0,
            Knight => 2,
            Bishop => 4,
            Rook => 6,
            Queen => 8,
            King => 10,
        }
    }

    pub fn as_fen_char(&self, color: Color) -> char {
        match (color, self) {
            (White, Pawn) => 'P',
            (White, Knight) => 'N',
            (White, Bishop) => 'B',
            (White, Rook) => 'R',
            (White, Queen) => 'Q',
            (White, King) => 'K',
            (Black, Pawn) => 'p',
            (Black, Knight) => 'n',
            (Black, Bishop) => 'b',
            (Black, Rook) => 'r',
            (Black, Queen) => 'q',
            (Black, King) => 'k',
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