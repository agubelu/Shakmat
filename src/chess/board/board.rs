use std::fmt::Display;
use std::result::Result;
use rayon::prelude::*;

use crate::chess::{CastlingRights, Color, PieceType, BitBoard, Move};
use crate::chess::fen::{read_fen, DEFAULT_FEN};
use crate::chess::Color::*;
use crate::chess::PieceType::*;
use super::movegen;

use super::super::position::Square;

#[derive(Debug, Clone, Copy)]
pub struct Board {
    castling_rights: CastlingRights,
    turn: Color,
    half_turns_til_50move_draw: u16,
    full_turns: u16,
    en_passant_target: BitBoard,
    white_pieces: Pieces,
    black_pieces: Pieces,
    all_whites: BitBoard,
    all_blacks: BitBoard,
    all_pieces: BitBoard,
    black_attacks: BitBoard,
    white_attacks: BitBoard,
}

#[derive(Debug, Clone, Copy)]
pub struct Pieces {
    pub pawns: BitBoard,
    pub rooks: BitBoard,
    pub knights: BitBoard,
    pub bishops: BitBoard, 
    pub queens: BitBoard,
    pub king: BitBoard,
}

impl Board {
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let fen_info = read_fen(fen)?;

        let mut board = Self {
            castling_rights: fen_info.castling_rights,
            turn: fen_info.turn,
            en_passant_target: fen_info.en_passant_square,
            half_turns_til_50move_draw: 100 - fen_info.halfmoves_since_capture,
            full_turns: fen_info.fullmoves_since_start,
            white_pieces: fen_info.white_pieces,
            black_pieces: fen_info.black_pieces,
            all_whites: BitBoard::default(),
            all_blacks: BitBoard::default(),
            all_pieces: BitBoard::default(),
            black_attacks: BitBoard::default(),
            white_attacks: BitBoard::default(), 
        };

        board.update_aux_bitboards();
        Ok(board)
    }

    pub fn make_move(&self, movement: Move, check_legality: bool) -> Result<Self, String> {
        if check_legality {
            // This move was received from the user, check that it is indeed legal
            // We do this by making sure it exists in the list of allowed moves
            // Even though we generate all the moves just to check, this is only
            // done for user-provided moves. The moves made by the engine when
            // it is analyzing a position bypass this check
            
            // TODO
        }
        // Copy the current board and make the changes on it
        let mut new_board = *self;

        // Perform the movement in question
        if matches!(movement, Move::LongCastle | Move::ShortCastle) {
            new_board.castle(&movement);
            // Castling calls move_piece twice, so the half-turn counter for
            // the 50 move rule is updated twice, that's why we must substract 1
            new_board.half_turns_til_50move_draw -= 1;
        } else {
            new_board.move_piece(&movement);
        }

        new_board.update_en_passant(&movement);

        // Update the current color to play and the number of total turns,
        // if black just moved
        new_board.turn = !self.turn;
        if new_board.turn == White {
            new_board.full_turns += 1;
        }

        new_board.update_aux_bitboards();
        Ok(new_board)
    }

    pub fn pseudolegal_moves(&self, color: Color) -> Vec<Move> {
        movegen::get_pseudolegal_moves(self, color)
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        match color {
            White => !(self.white_pieces.king & self.black_attacks).is_empty(),
            Black => !(self.black_pieces.king & self.white_attacks).is_empty()
        }
    }

    pub fn ep_square(&self) -> BitBoard {
        self.en_passant_target
    }

    pub fn castling_info(&self) -> &CastlingRights {
        &self.castling_rights
    }

    pub fn turn_number(&self) -> u16 {
        self.full_turns
    }

    pub fn turn_color(&self) -> Color {
        self.turn
    }

    pub fn get_pieces(&self, color: Color) -> &Pieces {
        match color {
            White => &self.white_pieces,
            Black => &self.black_pieces
        }
    }

    pub fn get_color_bitboard(&self, color: Color) -> BitBoard {
        match color {
            White => self.all_whites,
            Black => self.all_blacks
        }
    }

    pub fn get_all_bitboard(&self) -> BitBoard {
        self.all_pieces
    }

    ///////////////////////////////////////////////////////////////////////////
    /// Private auxiliary functions

    fn move_piece(&mut self, movement: &Move) {
        // This function is called with legal moves, so we can assume
        // that the piece exists in the "from" position and can move to the
        // target position. It only does single moves, not castling.
        // It is also assumed that it is only called by normal or promotion
        // moves, which implement .from() and .to()
        let from_bb = BitBoard::new(1 << movement.from());
        let to_bb = BitBoard::new(1 << movement.to());
        let (moving_color, enemy_color) = (self.turn_color(), !self.turn_color());
        let enemy_pieces = self.get_color_bitboard(enemy_color);

        let mut is_capture = false;

        // If there is a piece in the destination square, remove it
        // First check for e.p., where the square we must remove is different
        if movement.is_ep() {
            let target_ep = match moving_color {
                White => movement.to() - 8,
                Black => movement.to() + 8,
            };
            let target_bb = BitBoard::new(1 << target_ep);
            *self.get_pieces_mut(enemy_color).get_pieces_of_type_mut(Pawn) ^= target_bb;
            is_capture = true;
        } else if !(enemy_pieces & to_bb).is_empty() {
            self.get_pieces_mut(enemy_color).apply_mask(!to_bb);
            is_capture = true;
        }

        // Move the piece, depending on whether this is a pawn promotion or not
        let our_pieces = self.get_pieces_mut(moving_color);
        if let Move::PawnPromotion { promote_to, ..} = movement {
            *our_pieces.get_pieces_of_type_mut(Pawn) ^= from_bb;
            *our_pieces.get_pieces_of_type_mut(*promote_to) ^= to_bb;
        } else {
            *our_pieces.get_pieces_of_type_mut(movement.piece_type()) ^= from_bb | to_bb;
        }

        // Update the counter towards the 50 move rule
        if is_capture || movement.piece_type() == Pawn {
            self.half_turns_til_50move_draw = 0;
        } else {
            self.half_turns_til_50move_draw += 1;
        }

        // Update castling rights
        self.update_castling_rights(movement);
    }

    fn castle(&mut self, movement: &Move) {
        // Note that "self.turn" still hasn't updated at this point, hence
        // we can use it to get which color is castling
        let color = self.turn_color();
        let short = matches!(movement, Move::ShortCastle);

        let row_start = if color == White { 0 } else { 56 };
        
        let (king_from, king_to, rook_from, rook_to) = if short {
            (row_start + 3, row_start + 1, row_start, row_start + 2)
        } else {
            (row_start + 3, row_start + 5, row_start + 7, row_start + 4)
        };

        let king_move = Move::Normal { from: king_from, to: king_to, ep: false, piece: King};
        let rook_move = Move::Normal { from: rook_from, to: rook_to, ep: false, piece: Rook};

        self.move_piece(&king_move);
        self.move_piece(&rook_move);
    }

    fn update_en_passant(&mut self, movement: &Move) {
        match movement {
            Move::Normal {piece: Pawn, from, to, ep: false } => {
                // This is done *before* the color is updated, hence,
                // the current turn is the one that played the move
                // Pawns move in increments (white) or decrements (black) of
                // 8, so we can use that to detect if it's a double push
                let color = self.turn_color();
                if color == White && to - from == 16 {
                    self.en_passant_target = BitBoard::new(*from as u64 + 8);
                } else if color == Black && from - to == 16 {
                    self.en_passant_target = BitBoard::new(*from as u64 - 8);
                } else {
                    self.en_passant_target.clear();
                }
            },
            _ => self.en_passant_target.clear(),
        };
    }

    fn update_castling_rights(&mut self, movement: &Move) {
        // Check if we are capturing one of the opponent's rooks and update
        // their castling rights
        let white_rooks = (7, 0);
        let black_rooks = (63, 56);

        let (from, to) = (movement.from(), movement.to());

        let color = self.turn_color();
        let op_color = !color;

        // Initial positions of the rooks of the color moving (0) and
        // the opposite color (1)
        let rook_positions = match color { // Queenside, kingside
            White => (white_rooks, black_rooks),
            Black => (black_rooks, white_rooks),
        };

        if self.castling_rights.can_castle_queenside(op_color) && to == rook_positions.1.0 {
            self.castling_rights.update_queenside(op_color, false);
        } else if self.castling_rights.can_castle_kingside(op_color) && to == rook_positions.1.1 {
            self.castling_rights.update_kingside(op_color, false);
        }

        // Check if we are moving our own king or one of our rooks
        if movement.piece_type() == King {
            self.castling_rights.disable_all(color);
        } else if self.castling_rights.can_castle_queenside(color) && from == rook_positions.0.0 {
            self.castling_rights.update_queenside(color, false);
        } else if self.castling_rights.can_castle_kingside(color) && from == rook_positions.0.1 {
            self.castling_rights.update_kingside(color, false);
        }
    }

    fn update_aux_bitboards(&mut self) {
        let blacks = self.black_pieces;
        let whites = self.white_pieces;
        self.all_blacks = blacks.pawns | blacks.rooks | blacks.knights | blacks.bishops | blacks.queens | blacks.king;
        self.all_whites = whites.pawns | whites.rooks | whites.knights | whites.bishops | whites.queens | whites.king;
        self.all_pieces = self.all_blacks | self.all_whites;

        self.white_attacks = movegen::get_controlled_squares(self, White);
        self.black_attacks = movegen::get_controlled_squares(self, Black);
    }

    pub fn get_pieces_mut(&mut self, color: Color) -> &mut Pieces {
        match color {
            White => &mut self.white_pieces,
            Black => &mut self.black_pieces
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        // The default FEN is hard-coded and correct, so we can unwrap the result safely
        Self::from_fen(DEFAULT_FEN).unwrap()
    }
}

impl Display for Board {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Dump the pieces from the bitboards into an 8x8 array
        let mut pieces: [[Option<char>; 8]; 8] = [[None; 8]; 8];

        for color in [Black, White].into_iter() {
            for piece_type in [King, Queen, Pawn, Knight, Bishop, Rook].into_iter() {
                let piece_bb = self.get_pieces(color).get_pieces_of_type(piece_type);
                // The pieces position atribute will be deprecated and it 
                // doesnt matter here
                for square in piece_bb.piece_indices() {
                    let bbsquare = Square::new(square as u8);
                    pieces[bbsquare.rank() as usize][bbsquare.file() as usize] = Some(piece_type.as_char(color));
                }
            }
        }

        // Print da thing
        writeln!(f, "{:?} to play, turn #{}\n", self.turn, self.full_turns)?;
        writeln!(f, "  ┌───┬───┬───┬───┬───┬───┬───┬───┐")?;

        for rank in (0..8).rev() {
            let pieces_line = (0..8)
                .map(|file| match pieces[rank][file] {
                    None => "   ".to_string(),
                    Some(c) => format!(" {} ", c)
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

impl Pieces {
    pub fn get_pieces_of_type(&self, piece_type: PieceType) -> BitBoard {
        match piece_type {
            Pawn => self.pawns,
            Knight => self.knights,
            Bishop => self.bishops,
            Rook => self.rooks,
            Queen => self.queens,
            King => self.king,
        }
    }

    pub fn get_pieces_of_type_mut(&mut self, piece_type: PieceType) -> &mut BitBoard {
        match piece_type {
            Pawn => &mut self.pawns,
            Knight => &mut self.knights,
            Bishop => &mut self.bishops,
            Rook => &mut self.rooks,
            Queen => &mut self.queens,
            King => &mut self.king,
        }
    }

    pub fn apply_mask(&mut self, mask: BitBoard) {
        self.pawns &= mask;
        self.knights &= mask;
        self.bishops &= mask;
        self.rooks &= mask;
        self.queens &= mask;
        self.king &= mask;
    }
}

impl Default for Pieces {
    fn default() -> Self {
        Self {
            pawns: BitBoard::default(),
            rooks: BitBoard::default(),
            knights: BitBoard::default(),
            bishops: BitBoard::default(),
            queens: BitBoard::default(),
            king: BitBoard::default(),
        }
    }
}