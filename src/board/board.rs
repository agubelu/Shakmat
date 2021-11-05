use std::fmt::Display;
use std::result::Result;

use super::Piece;
use crate::game_elements::{Color, Position, CastlingRights};
use crate::fen::{read_fen, DEFAULT_FEN};

pub type BoardSquares = [[Option<Piece>; 8]; 8];

pub struct Board {
    castling_rights: CastlingRights,
    en_passant_target: Option<Position>,
    turn: Color,
    //white_king_pos: Position,
    //black_king_pos: Position,
    squares: BoardSquares,
    half_turns_til_50move_draw: u16,
    full_turns: u16,
}

impl Board {
    pub fn from_fen(fen: &str) -> Result<Board, String> {
        let fen_info = read_fen(fen)?;
        let board = Board {
            squares: fen_info.board_state,
            castling_rights: fen_info.castling_rights,
            turn: fen_info.turn,
            en_passant_target: fen_info.en_passant_square,
            half_turns_til_50move_draw: 100 - fen_info.halfmoves_since_capture,
            full_turns: fen_info.fullmoves_since_start,
        };

        Ok(board)
    }

    pub fn default() -> Board {
        // The default FEN is hard-coded and correct,
        // so we can unwrap it safely
        Board::from_fen(DEFAULT_FEN).unwrap()
    }
}

impl Display for Board {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{:?} to play, turn #{}\n", self.turn, self.full_turns)?;

        writeln!(f, "  ┌───┬───┬───┬───┬───┬───┬───┬───┐")?;

        for rank in (0..8).rev() {
            let pieces_line = (0..8)
                .map(|file| self.squares[rank][file])
                .map(|sqre| match sqre {
                    None => "   ".to_string(),
                    Some(piece) => format!(" {} ", piece.as_char().to_string())
                })
                .collect::<Vec<String>>()
                .join("│");

            writeln!(f, "{} │{}│", rank + 1, pieces_line)?;

            if rank != 0 {
                writeln!(f, "  ├───┼───┼───┼───┼───┼───┼───┼───┤")?;
            }
        }


        writeln!(f, "  └───┴───┴───┴───┴───┴───┴───┴───┘")?;
        writeln!(f, "    a   b   c   d   e   f   g   h ")?;
        Ok(())
    }
    
}