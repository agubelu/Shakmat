use std::result::Result;

use crate::PieceType;
use crate::board::{Board, BitBoard, Pieces};
use crate::game_elements::{Color::*, PieceType::*, CastlingRights, Color, Square};

pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub struct FENInfo {
    pub turn: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_square: BitBoard,
    pub halfmoves_since_capture: u16,
    pub fullmoves_since_start: u16,
    pub black_pieces: Pieces,
    pub white_pieces: Pieces,
    pub piece_on_square: [Option<PieceType>; 64],
}

pub fn read_fen(fen: &str) -> Result<FENInfo, String> {
    let fen_parts: Vec<&str> = fen.split_whitespace().collect();

    if fen_parts.len() != 6 {
        return Err("The provided FEN must have 6 parts".to_string());
    }

    let mut fen_info = FENInfo {
        turn: Color::White,
        castling_rights: CastlingRights::none(),
        en_passant_square: BitBoard::new(0),
        halfmoves_since_capture: 0,
        fullmoves_since_start: 0,
        black_pieces: Pieces::default(),
        white_pieces: Pieces::default(),
        piece_on_square: [None; 64]
    };

    // Load the current board state, return an error if we find an unexpected character
    load_board(fen_parts[0], &mut fen_info)?;

    // Load the current turn
    fen_info.turn = match fen_parts[1] {
        "w" => White,
        "b" => Black,
         x => return Err(format!("The turn '{x}' provided in the FEN is invalid")),
    };

    // Load castling rights
    load_castling(fen_parts[2], &mut fen_info)?;

    // Load en passant square, if any
    if fen_parts[3] != "-" {
        fen_info.en_passant_square = Square::from_notation(fen_parts[3])?.as_bitboard();
    }

    // Load halfmoves since capture and fullmoves since start
    fen_info.halfmoves_since_capture = fen_parts[4].parse().map_err(|_| "Halfmoves since capture is not a valid number")?;
    fen_info.fullmoves_since_start = fen_parts[5].parse().map_err(|_| "Full moves since start is not a valid number")?;

    Ok(fen_info)
}

pub fn create_fen(board: &Board) -> String {
    let pos = (0..8).rev().map(|rank| get_rank_fen(board, rank)).collect::<Vec<_>>().join("/");
    let turn = if board.turn_color() == White { "w" } else { "b" };
    let castling = save_castling(board.castling_info());
    let ep = if board.ep_square().is_empty() {
        "-".to_owned()
    } else {
        Square::new(board.ep_square().first_piece_index()).to_string()
    };

    let fifty_rule = board.fifty_move_rule_counter().to_string();
    let turn_count = board.turn_number().to_string();

    format!("{pos} {turn} {castling} {ep} {fifty_rule} {turn_count}")
}

// Creates a FEN representation of the given rank for a board
fn get_rank_fen(board: &Board, rank: usize) -> String {
    let mut res = String::new();
    let mut empty = 0;

    // Traverse the rank left to right and create the string
    for i in (0..8).rev() {
        let square = 8 * rank + i;

        if let Some(piece) = board.piece_on(square as u8) {
            let mask = 1 << square;
            let color = if (board.get_color_bitboard(White) & mask).is_empty() {
                Black
            } else {
                White
            };

            // Write all previous spaces and the current piece
            if empty != 0 {
                res.push_str(&empty.to_string());
                empty = 0;
            }
            res.push(piece.as_fen_char(color));
        } else {
            // Empty space, increment the counter
            empty += 1;
        }
    }

    // Finally, as per FEN definition, write the trailing empty squares
    if empty != 0 {
        res.push_str(&empty.to_string());
    }

    res
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
            let is_digit = ch.is_ascii_digit();

            if is_digit {
                file += ch.to_digit(10).unwrap() as usize;
            } else {
                let bb = Square::from_file_rank(file as u8, rank as u8)?.as_bitboard();
                let (color, piece) = match ch {
                    'r' => (Black, Rook),
                    'n' => (Black, Knight),
                    'b' => (Black, Bishop),
                    'q' => (Black, Queen),
                    'k' => (Black, King),
                    'p' => (Black, Pawn),
                    'R' => (White, Rook),
                    'N' => (White, Knight),
                    'B' => (White, Bishop),
                    'Q' => (White, Queen),
                    'K' => (White, King),
                    'P' => (White, Pawn),
                     _  if is_digit => continue, // Already handled
                     _  => return Err(format!("Invalid character '{ch}' while reading the board state from FEN"))
                };

                let pieces = match color {
                    White => &mut fen_info.white_pieces,
                    Black => &mut fen_info.black_pieces,
                };

                *pieces.get_pieces_of_type_mut(piece) |= bb;
                let square = bb.first_piece_index();
                fen_info.piece_on_square[square as usize] = Some(piece);

                file += 1;
            }
        }
    }

    if fen_info.white_pieces.get_pieces_of_type(King).is_empty() {
        return Err("White must have a king!".to_owned());
    } else if fen_info.black_pieces.get_pieces_of_type(King).is_empty() {
        return Err("Black must have a king!".to_owned());
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
             x  => return Err(format!("Invalid chracter '{x}' while reading castling info from FEN"))
        }
    }

    Ok(())
}

fn save_castling(castling_info: &CastlingRights) -> String {
    if castling_info.has_no_rights() {
        return "-".to_owned();
    }

    let mut res = String::new();
    for color in [White, Black] {
        if castling_info.can_castle_kingside(color) {
            res.push(if color == White {'K'} else {'k'});
        }

        if castling_info.can_castle_queenside(color) {
            res.push(if color == White {'Q'} else {'q'});
        }
    }

    res
}