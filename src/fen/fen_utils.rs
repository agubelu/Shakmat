use std::result::Result;

use crate::game_elements::{Color, CastlingRights, Position, PieceType};
use crate::game_elements::{Color::*, PieceType::*};
use crate::board::{Piece, BoardSquares};

pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub struct FENInfo {
    pub board_state: BoardSquares,
    pub turn: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_square: Option<Position>,
    pub halfmoves_since_capture: u16,
    pub fullmoves_since_start: u16,
}

pub fn read_fen(fen: &str) -> Result<FENInfo, String> {
    let fen_parts: Vec<&str> = fen.split(' ').collect();

    if fen_parts.len() != 6 {
        return Err("The provided FEN must have 6 parts".to_string());
    }

    let mut fen_info = FENInfo {
        board_state: [[None; 8]; 8],
        turn: Color::White,
        castling_rights: CastlingRights::none(),
        en_passant_square: None,
        halfmoves_since_capture: 0,
        fullmoves_since_start: 0,
    };

    // Load the current board state, return an error if we find an unexpected character
    load_board(fen_parts[0], &mut fen_info)?;

    // Load the current turn
    fen_info.turn = match fen_parts[1] {
        "w" => White,
        "b" => Black,
         x => return Err(format!("The turn '{}' provided in the FEN is invalid", x)),
    };

    // Load castling rights
    load_castling(fen_parts[2], &mut fen_info)?;

    // Load en passant square, if any
    if fen_parts[3] != "-" {
        fen_info.en_passant_square = Some(Position::from_notation(fen_parts[3])?);
    }

    // Load halfmoves since capture and fullmoves since start
    fen_info.halfmoves_since_capture = fen_parts[4].parse().map_err(|_| "Halfmoves since capture is not a valid number")?;
    fen_info.fullmoves_since_start = fen_parts[5].parse().map_err(|_| "Full moves since start is not a valid number")?;

    Ok(fen_info)
}

fn load_board(board_info: &str, fen_info: &mut FENInfo) -> Result<(), String> {
    let rows: Vec<&str> = board_info.split('/').collect();

    if rows.len() != 8 {
        return Err("The board must have 8 rows".to_string());
    }

    for (row_i, row_info) in rows.iter().enumerate() {
        let rank = 7 - row_i;
        let mut file = 0;
        for ch in row_info.chars() {
            let is_digit = ch.is_digit(10);

            if is_digit {
                file += ch.to_digit(10).unwrap() as usize;
            } else {
                match ch {
                    'r' => fen_info.board_state[rank][file] = Some(Piece::new(Black, Rook)),
                    'n' => fen_info.board_state[rank][file] = Some(Piece::new(Black, Knight)),
                    'b' => fen_info.board_state[rank][file] = Some(Piece::new(Black, Bishop)),
                    'q' => fen_info.board_state[rank][file] = Some(Piece::new(Black, Queen)),
                    'k' => fen_info.board_state[rank][file] = Some(Piece::new(Black, King)),
                    'p' => fen_info.board_state[rank][file] = Some(Piece::new(Black, Pawn)),
                    'R' => fen_info.board_state[rank][file] = Some(Piece::new(White, Rook)),
                    'N' => fen_info.board_state[rank][file] = Some(Piece::new(White, Knight)),
                    'B' => fen_info.board_state[rank][file] = Some(Piece::new(White, Bishop)),
                    'Q' => fen_info.board_state[rank][file] = Some(Piece::new(White, Queen)),
                    'K' => fen_info.board_state[rank][file] = Some(Piece::new(White, King)),
                    'P' => fen_info.board_state[rank][file] = Some(Piece::new(White, Pawn)),
                     _  if is_digit => {}, // Already handled
                     _  => return Err(format!("Invalid character '{}' while reading the board state from FEN", ch))
                }

                file += 1;
            }
        }
    }

    Ok(())
}

fn load_castling(castling_info: &str, fen_info: &mut FENInfo) -> Result<(), String> {
    // The castling rights are all initially set to false
    for ch in castling_info.chars() {
        match ch {
            'K' => fen_info.castling_rights.update_kingside(White, true),
            'Q' => fen_info.castling_rights.update_queenside(White, true),
            'k' => fen_info.castling_rights.update_kingside(Black, true),
            'q' => fen_info.castling_rights.update_queenside(Black, true),
            '-' => {},
             x  => return Err(format!("Invalid chracter '{}' while reading castling info from FEN", x))
        }
    }

    Ok(())
}