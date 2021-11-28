use std::default;
use std::fmt::Display;
use std::result::Result;
use rayon::prelude::*;

// TODO tidy up these imports

use crate::chess::game_elements::position::{DOWN, UP};
use crate::chess::{CastlingRights, Color, Move, Piece, PieceType, Position, BitBoard, BBMove};
use crate::chess::fen::{read_fen, DEFAULT_FEN};
use crate::chess::Color::*;
use crate::chess::PieceType::*;
use super::movegen;

use super::super::position::BBSquare;
use crate::magic;

const PIECE_TYPES: [PieceType; 6] = [King, Queen, Bishop, Knight, Rook, Pawn];
const COLORS: [Color; 2] = [Black, White];

pub type PieceArray = [Option<Piece>; 16];
type BoardSquares = [[Option<PieceArrayPos>; 8]; 8];


#[derive(Debug, Clone, Copy)]
pub struct Board {
    castling_rights: CastlingRights,
    en_passant_target: Option<Position>,
    turn: Color,
    half_turns_til_50move_draw: u16,
    full_turns: u16,
    squares: BoardSquares,
    white_pieces: PieceArray,
    black_pieces: PieceArray,
}

#[derive(Debug, Clone, Copy)]
pub struct BBBoard {
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

impl Pieces {
    pub fn get_pieces_of_type(&self, piece_type: PieceType) -> BitBoard {
        // Especially useful when printing the board to the console
        match piece_type {
            Pawn => self.pawns,
            Knight => self.knights,
            Bishop => self.bishops,
            Rook => self.rooks,
            Queen => self.queens,
            King => self.king,
        }
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

impl BBBoard {
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        // TO-DO: Adapt the FEN thingy to bitboards so this is smaller
        let fen_info = read_fen(fen)?;
        let mut board = Self {
            castling_rights: fen_info.castling_rights,
            turn: fen_info.turn,
            en_passant_target: BitBoard::default(),
            half_turns_til_50move_draw: 100 - fen_info.halfmoves_since_capture,
            full_turns: fen_info.fullmoves_since_start,
            white_pieces: Pieces::default(),
            black_pieces: Pieces::default(),
            all_whites: BitBoard::default(),
            all_blacks: BitBoard::default(),
            all_pieces: BitBoard::default(),
            black_attacks: BitBoard::default(),
            white_attacks: BitBoard::default(), 
        };

        for white_piece in fen_info.white_pieces.into_iter().flatten() {
            let pos = white_piece.position();
            let bb = BBSquare::from_file_rank(pos.file_u() as u8, pos.rank_u() as u8).unwrap().as_bitboard();
            match white_piece.piece_type() {
                King => board.white_pieces.king |= bb,
                Queen => board.white_pieces.queens |= bb,
                Rook => board.white_pieces.rooks |= bb,
                Bishop => board.white_pieces.bishops |= bb,
                Knight => board.white_pieces.knights |= bb,
                Pawn => board.white_pieces.pawns |= bb,
            }
        }

        for black_piece in fen_info.black_pieces.into_iter().flatten() {
            let pos = black_piece.position();
            let bb = BBSquare::from_file_rank(pos.file_u() as u8, pos.rank_u() as u8).unwrap().as_bitboard();
            match black_piece.piece_type() {
                King => board.black_pieces.king |= bb,
                Queen => board.black_pieces.queens |= bb,
                Rook => board.black_pieces.rooks |= bb,
                Bishop => board.black_pieces.bishops |= bb,
                Knight => board.black_pieces.knights |= bb,
                Pawn => board.black_pieces.pawns |= bb,
            }
        }

        board.update_aux_bitboards();
        board.update_attacks(!board.turn);
        Ok(board)
    }

    pub fn pseudolegal_moves(&self, color: Color) -> Vec<BBMove> {
        movegen::get_pseudolegal_moves(self, color)
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        match color {
            White => !(self.white_pieces.king & self.black_attacks).is_empty(),
            Black => !(self.black_pieces.king & self.white_attacks).is_empty()
        }
    }

    pub fn get_en_passant_target(&self) -> BitBoard {
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

    fn update_aux_bitboards(&mut self) {
        let blacks = self.black_pieces;
        let whites = self.white_pieces;
        self.all_blacks = blacks.pawns | blacks.rooks | blacks.knights | blacks.bishops | blacks.queens | blacks.king;
        self.all_whites = whites.pawns | whites.rooks | whites.knights | whites.bishops | whites.queens | whites.king;
        self.all_pieces = self.all_blacks | self.all_whites;
    }

    fn update_attacks(&mut self, color: Color) {
        // TODO
    }
}

#[derive(Debug, Clone, Copy)]
struct PieceArrayPos {
    pub color: Color,
    pub index: usize,
}

impl Board {
    pub fn from_fen(fen: &str) -> Result<Board, String> {
        let fen_info = read_fen(fen)?;
        let mut board = Board {
            castling_rights: fen_info.castling_rights,
            turn: fen_info.turn,
            en_passant_target: fen_info.en_passant_square,
            half_turns_til_50move_draw: 100 - fen_info.halfmoves_since_capture,
            full_turns: fen_info.fullmoves_since_start,
            squares: [[None; 8]; 8],
            white_pieces: fen_info.white_pieces,
            black_pieces: fen_info.black_pieces
        };

        // Initialize the square reference info
        for (i, piece_opt) in board.white_pieces.iter().enumerate() {
            if let Some(piece) = piece_opt {
                let pos = piece.position();
                board.squares[pos.rank_u()][pos.file_u()] = Some(PieceArrayPos {
                    color: Color::White,
                    index: i
                });
            }
        }

        for (i, piece_opt) in board.black_pieces.iter().enumerate() {
            if let Some(piece) = piece_opt {
                let pos = piece.position();
                board.squares[pos.rank_u()][pos.file_u()] = Some(PieceArrayPos {
                    color: Color::Black,
                    index: i
                });
            }
        }

        Ok(board)
    }

    pub fn make_move(&self, movement: Move, check_legality: bool) -> Result<Board, String> {
        if check_legality {
            // This move was received from the user, check that it is indeed legal
            // We do this by making sure it exists in the list of allowed moves
            // Even though we generate all the moves just to check, this is only
            // done for user-provided moves. The moves made by the engine when
            // it is analyzing a position bypass this check
            if !self.get_current_turn_moves().contains(&movement) {
                return Err("Illegal move".to_owned())
            }
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
        if new_board.turn == Color::White {
            new_board.full_turns += 1;
        }

        Ok(new_board)
    }

    pub fn get_current_turn_moves(&self) -> Vec<Move> {
        self.get_pieces(self.turn)
            .iter()
            .filter_map(|&p| p)
            .flat_map(|piece| piece.get_legal_moves(self).into_iter())
            .collect()
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        self.get_king_position(color).is_attacked_by(self, !color)
    }

    pub fn get_pos(&self, pos: &Position) -> Option<&Piece> {
        self.get_piece_array_info(pos).as_ref()
            .map(|arr_info| self.get_pieces(arr_info.color)[arr_info.index].as_ref().unwrap())

    }

    pub fn get_en_passant_target(&self) -> &Option<Position> {
        &self.en_passant_target
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

    pub fn perft(&self, depth: u16) -> u64 {
        self._perft(depth, true)
    }

    ///////////////////////////////////////////////////////////////////////////
    /// Aux functions to help with moves
    fn move_piece(&mut self, movement: &Move) {
        // This function is called with legal moves, so we can assume
        // that the piece exists in the "from" position and can move to the
        // target position. It only does single moves, not castling
        let from = movement.from();
        let to = movement.to();

        // If there is a piece in the destination square, remove it
        let is_capture = self.remove_piece(to);

        // Update the position of the piece that is moving
        let from_data = self.get_piece_array_info(from).unwrap();
        let piece_info = self.get_pieces(from_data.color)[from_data.index].as_ref().unwrap();

        // Store this to avoid holding a reference
        let piece_type = piece_info.piece_type();
        let piece_color = piece_info.color();

        // If this is a pawn capturing en passant, the piece to remove is
        // actually behind it
        if let Some(ep_target) = self.get_en_passant_target() {
            if *ep_target == *to && piece_type == PieceType::Pawn {
                let diff = match piece_color {
                    Color::White => DOWN,
                    Color::Black => UP,
                };

                let pos_to_delete = to.add_delta(&diff);
                self.remove_piece(&pos_to_delete);
            }
        }

        // Move the piece
        self.squares[to.rank_u()][to.file_u()] = Some(from_data);
        self.squares[from.rank_u()][from.file_u()] = None;

        // Update the piece's position in place
        let piece = self.get_pieces_mut(from_data.color)[from_data.index].as_mut().unwrap();
        piece.update_position(*to);
        // If this is a promotion, change the piece's type
        if let Move::PawnPromotion{ promote_to: dest_type , ..} = movement {
            piece.update_type(*dest_type);
        }

        // Update the counter towards the 50 move rule
        if is_capture || piece_type == PieceType::Pawn {
            self.half_turns_til_50move_draw = 0;
        } else {
            self.half_turns_til_50move_draw += 1;
        }

        // Update castling rights
        self.update_castling_rights(piece_color, piece_type, from, to);
    }

    fn update_en_passant(&mut self, movement: &Move) {
        // Set or disable the e.p. target square
        // Note that this is done after the piece has already been moved,
        // so it is currently in the "to" square
        let ep = match movement {
            Move::NormalMove { from, to } => {
                let piece = self.get_pos(to).unwrap(); // The piece is guaranteed to be there
                if piece.piece_type() == PieceType::Pawn && (from.rank - to.rank).abs() == 2 {
                    // It is a pawn that has moved 2 squares, therefore, 
                    // it can be capture en passant. Determine the target square
                    let diff = match piece.color() {
                        Color::White => DOWN,
                        Color::Black => UP,
                    };
                    Some(to.add_delta(&diff))
                } else {
                    None
                }
            },
            _ => None,
        };

        self.en_passant_target = ep;
    }

    fn castle(&mut self, movement: &Move) {
        // Note that "self.turn" still hasn't updated at this point, hence
        // we can use it to get which color is castling
        let rank = if self.turn == Color::White {0} else {7};
        let file_king_from = 4;
        let file_king_to = if matches!(movement, Move::ShortCastle) {6} else {2};
        let file_rook_from = if matches!(movement, Move::ShortCastle) {7} else {0};
        let file_rook_to = if matches!(movement, Move::ShortCastle) {5} else {3};

        let pos_king_from = Position::new_0based(file_king_from, rank);
        let pos_king_to = Position::new_0based(file_king_to, rank);

        let pos_rook_from = Position::new_0based(file_rook_from, rank);
        let pos_rook_to = Position::new_0based(file_rook_to, rank);

        self.move_piece(&Move::NormalMove{from: pos_king_from, to: pos_king_to});
        self.move_piece(&Move::NormalMove{from: pos_rook_from, to: pos_rook_to});
    }

    fn update_castling_rights(&mut self, color: Color, piece_type: PieceType, from: &Position, to: &Position) {
        // Check if we are capturing one of the opponent's rooks and update
        // their castling rights
        let white_rooks = ((0, 0), (7, 0));
        let black_rooks = ((0, 7), (7, 7));

        let op_color = !color;

        // Initial positions of the rooks of the color moving (0) and
        // the opposite color (1)
        let rook_positions = match color { // Queenside, kingside
            Color::White => (white_rooks, black_rooks),
            Color::Black => (black_rooks, white_rooks),
        };

        if self.castling_rights.can_castle_queenside(op_color) && to == rook_positions.1.0 {
            self.castling_rights.update_queenside(op_color, false);
        } else if self.castling_rights.can_castle_kingside(op_color) && to == rook_positions.1.1 {
            self.castling_rights.update_kingside(op_color, false);
        }

        // Check if we are moving our own king or one of our rooks
        if piece_type == PieceType::King {
            self.castling_rights.disable_all(color);
        } else if self.castling_rights.can_castle_queenside(color) && from == rook_positions.0.0 {
            self.castling_rights.update_queenside(color, false);
        } else if self.castling_rights.can_castle_kingside(color) && from == rook_positions.0.1 {
            self.castling_rights.update_kingside(color, false);
        }
    }

    fn get_pieces(&self, color: Color) -> &PieceArray {
        match color {
            Color::White => &self.white_pieces,
            Color::Black => &self.black_pieces
        }
    }

    fn get_king_position(&self, color: Color) -> Position {
        // The king is guaranteed to exist and to be in the
        // first position of the piece array, hence, we can unwrap it safely
        *self.get_pieces(color)[0].unwrap().position()
    }

    fn remove_piece(&mut self, pos: &Position) -> bool {
        let pos_data = self.squares[pos.rank_u()][pos.file_u()];
        if let Some(tile_info) = pos_data {
            self.get_pieces_mut(tile_info.color)[tile_info.index] = None;
            self.squares[pos.rank_u()][pos.file_u()] = None;
            return true;
        }

        false
    }

    fn get_pieces_mut(&mut self, color: Color) -> &mut PieceArray {
        match color {
            Color::White => &mut self.white_pieces,
            Color::Black => &mut self.black_pieces
        }
    }

    fn get_piece_array_info(&self, pos: &Position) -> &Option<PieceArrayPos> {
        &self.squares[pos.rank_u()][pos.file_u()]
    }

    fn get_current_turn_pseudolegal_moves(&self) -> Vec<Move> {
        self.get_pieces(self.turn)
            .iter()
            .filter_map(|&p| p)
            .flat_map(|piece| piece.get_pseudolegal_moves(self).into_iter())
            .collect()
    }

    fn _perft(&self, depth: u16, multithread: bool) -> u64 {
        if depth == 1 {
            return self.get_current_turn_moves().len() as u64
        }

        let pseudo_moves = self.get_current_turn_pseudolegal_moves();

        if multithread {
            pseudo_moves.into_par_iter().filter_map(|mv| {
                let new_board = self.make_move(mv, false).unwrap();
                if matches!(mv, Move::LongCastle | Move::ShortCastle) || !new_board.is_in_check(self.turn_color()) {
                    Some(new_board._perft(depth - 1, false))
                } else {
                    None
                }
            }).sum()
        } else {
            pseudo_moves.into_iter().filter_map(|mv| {
                let new_board = self.make_move(mv, false).unwrap();
                if matches!(mv, Move::LongCastle | Move::ShortCastle) || !new_board.is_in_check(self.turn_color()) {
                    Some(new_board._perft(depth - 1, false))
                } else {
                    None
                }
            }).sum()
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        // The default FEN is hard-coded and correct, so we can unwrap it safely
        Board::from_fen(DEFAULT_FEN).unwrap()
    }
}

impl Display for Board {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{:?} to play, turn #{}\n", self.turn, self.full_turns)?;
        writeln!(f, "  ┌───┬───┬───┬───┬───┬───┬───┬───┐")?;

        for rank in (0..8).rev() {
            let pieces_line = (0..8)
                .map(|file| Position::new_0based(file, rank))
                .map(|sqre| match self.get_pos(&sqre) {
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

impl Default for BBBoard {
    fn default() -> Self {
        // The default FEN is hard-coded and correct, so we can unwrap the result safely
        Self::from_fen(DEFAULT_FEN).unwrap()
    }
}

impl Display for BBBoard {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Dump the pieces from the bitboards into an 8x8 array
        let mut pieces: [[Option<Piece>; 8]; 8] = [[None; 8]; 8];

        for color in COLORS.into_iter() {
            for piece_type in PIECE_TYPES.into_iter() {
                let piece_bb = self.get_pieces(color).get_pieces_of_type(piece_type);
                let piece = Piece::new(color, piece_type, Position::new_0based(0, 0));
                // The pieces position atribute will be deprecated and it 
                // doesnt matter here
                for square in piece_bb.piece_indices() {
                    let bbsquare = BBSquare::new(square as u8);
                    pieces[bbsquare.rank() as usize][bbsquare.file() as usize] = Some(piece);
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