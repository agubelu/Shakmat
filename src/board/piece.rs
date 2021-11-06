use std::fmt::{Display, Result};

use crate::game_elements::{Color::*, PieceType::*};
use crate::game_elements::{Color, PieceType, Position, Move};
use crate::board::Board;

#[derive(Debug, Clone, Copy)]
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
}

impl Piece {
    pub fn new(color: Color, piece_type: PieceType) -> Self {
        Piece { color, piece_type }
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn piece_type(&self) -> PieceType {
        self.piece_type
    }

    pub fn get_pseudolegal_moves(&self, pos: &Position, board: &Board) -> Vec<Move> {
        match self.piece_type {
            King => self.get_moves_king(pos, board),
            Knight => self.get_moves_knight(pos, board),
            _ => unimplemented!()
        }
    }

    pub fn get_legal_moves(&self, pos: &Position, board: &Board) -> Vec<Move> {
        self.get_pseudolegal_moves(pos, board)
            .into_iter()
            .filter(|&m| !board.make_move(m, false).is_check(self.color))
            .collect()
    }

    pub fn as_char(&self) -> char {
        match (self.color, self.piece_type) {
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

    ///////////////////////////////////////////////////////////////////////////
    /// Move functions for the diferent pieces
    /// Maybe transform Piece into a trait and have different pieces implement
    /// it? Im not sure how that would perform in terms of efficiency
    fn get_moves_king(&self, pos: &Position, board: &Board) -> Vec<Move> {
        // TODO: castling
        pos.king_moves()
            .iter()
            .filter(|&future_pos| {
                let future_square = board.get_pos(future_pos);
                future_square.is_none() || future_square.unwrap().color != self.color
            })
            .map(|future_pos| Move::NormalMove { from: *pos, to: *future_pos })
            .collect()
    }

    fn get_moves_knight(&self, pos: &Position, board: &Board) -> Vec<Move> {
        pos.knight_jumps()
            .iter()
            .filter(|&future_pos| {
                let future_square = board.get_pos(future_pos);
                future_square.is_none() || future_square.unwrap().color != self.color
            })
            .map(|future_pos| Move::NormalMove { from: *pos, to: *future_pos })
            .collect()
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result {
        writeln!(f, "{}", self.as_char())
    }
}