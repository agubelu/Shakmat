use std::result::Result;

use crate::chess::{Color, CastlingRights, Position, Piece, PieceArray};
use crate::chess::{Color::*, PieceType::*};
use crate::chess::position::CoordElem;

pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub struct FENInfo {
    pub turn: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_square: Option<Position>,
    pub halfmoves_since_capture: u16,
    pub fullmoves_since_start: u16,
    pub black_pieces: PieceArray,
    pub white_pieces: PieceArray
}

pub fn read_fen(fen: &str) -> Result<FENInfo, String> {
    let fen_parts: Vec<&str> = fen.split(' ').collect();

    if fen_parts.len() != 6 {
        return Err("The provided FEN must have 6 parts".to_string());
    }

    let mut fen_info = FENInfo {
        turn: Color::White,
        castling_rights: CastlingRights::none(),
        en_passant_square: None,
        halfmoves_since_capture: 0,
        fullmoves_since_start: 0,
        black_pieces: [None; 16],
        white_pieces: [None; 16]
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

    // Store the pieces in vectors, which will then be dumped into the piece array
    let mut white_pieces = Vec::with_capacity(16);
    let mut black_pieces = Vec::with_capacity(16);

    for (row_i, row_info) in rows.iter().enumerate() {
        let rank = 7 - row_i;
        let mut file = 0;
        for ch in row_info.chars() {
            let is_digit = ch.is_digit(10);

            if is_digit {
                file += ch.to_digit(10).unwrap() as usize;
            } else {
                let pos = Position::new_0based(file as CoordElem, rank as CoordElem);
                let piece = match ch {
                    'r' => Piece::new(Black, Rook, pos),
                    'n' => Piece::new(Black, Knight, pos),
                    'b' => Piece::new(Black, Bishop, pos),
                    'q' => Piece::new(Black, Queen, pos),
                    'k' => Piece::new(Black, King, pos),
                    'p' => Piece::new(Black, Pawn, pos),
                    'R' => Piece::new(White, Rook, pos),
                    'N' => Piece::new(White, Knight, pos),
                    'B' => Piece::new(White, Bishop, pos),
                    'Q' => Piece::new(White, Queen, pos),
                    'K' => Piece::new(White, King, pos),
                    'P' => Piece::new(White, Pawn, pos),
                     _  if is_digit => continue, // Already handled
                     _  => return Err(format!("Invalid character '{}' while reading the board state from FEN", ch))
                };

                // Insert it into the corresponding vec
                let piece_vec = match piece.color() {
                    White => &mut white_pieces,
                    Black => &mut black_pieces,
                };

                // Ensure that the king is always in the first position
                if piece.piece_type() == King {
                    piece_vec.insert(0, piece);
                } else {
                    piece_vec.push(piece);
                }

                file += 1;
            }
        }
    }

    // Check that both colors have at least one king
    if white_pieces.is_empty() || white_pieces[0].piece_type() != King {
        return Err("White must have at least one king".to_owned());
    }

    if black_pieces.is_empty() || black_pieces[0].piece_type() != King {
        return Err("Black must have at least one king".to_owned());
    }

    // Current limitation: the piece array in the board only supports
    // 16 pieces per side
    if black_pieces.len() > 16 || white_pieces.len() > 16 {
        return Err("Due to current limitations, only a maximum of 16 pieces per side are supported.".to_owned())
    }

    // Everything's fine, dump the pieces into the arrays
    for (i, piece) in white_pieces.drain(..).enumerate() {
        fen_info.white_pieces[i] = Some(piece);
    }

    for (i, piece) in black_pieces.drain(..).enumerate() {
        fen_info.black_pieces[i] = Some(piece);
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