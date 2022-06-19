use std::fmt::Display;
use std::result::Result;
use rayon::prelude::*;

use crate::game_elements::{CastlingRights, Color, Color::*, PieceType, PieceType::*, Move, Square};
use crate::board::BitBoard;
use crate::fen::{read_fen, DEFAULT_FEN};
use crate::zobrist;
use crate::magic::EP_ATTACKS;
use super::movegen;

// Struct to hold info about the things that change between moves and that are
// slow to recompute. This is only used internally when making/unmaking moves
#[derive(Copy, Clone)]
struct State {
    castling_rights: CastlingRights,
    fifty_move_rule_counter: u16,
    en_passant_target: BitBoard,
    last_moved: u8,
    black_attacks: BitBoard,
    white_attacks: BitBoard,
    zobrist_key: u64,
}

// This holds the needed info to quickly undo a move
#[derive(Copy, Clone)]
struct MoveUndoData {
    state: State,
    captured_piece: Option<PieceType>,
    ep: bool
}

#[derive(Clone)]
pub struct Board {
    state: State,
    previous_moves: Vec<MoveUndoData>,
    turn: Color,
    full_turns: u16,
    plies: u16,
    white_pieces: Pieces,
    black_pieces: Pieces,
    all_whites: BitBoard,
    all_blacks: BitBoard,
    all_pieces: BitBoard,
    piece_on_square: [Option<PieceType>; 64],
}

#[derive(Clone, Copy, Default)]
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
        let plies = (fen_info.fullmoves_since_start - 1) * 2 
            + (fen_info.turn == Black) as u16;


        let all_blacks = fen_info.black_pieces.combine();
        let all_whites = fen_info.white_pieces.combine();
        let all_pieces = all_blacks | all_whites;

        let state = State {
            castling_rights: fen_info.castling_rights,
            en_passant_target: fen_info.en_passant_square,
            fifty_move_rule_counter: fen_info.halfmoves_since_capture,
            black_attacks: BitBoard::default(),
            white_attacks: BitBoard::default(),
            last_moved: u8::MAX, // We don't know which piece was the last to move
            zobrist_key: 0,
        };

        let mut board = Self {
            plies,
            state,
            all_whites,
            all_blacks,
            all_pieces,
            previous_moves: Vec::with_capacity(250),
            turn: fen_info.turn,
            full_turns: fen_info.fullmoves_since_start,
            white_pieces: fen_info.white_pieces,
            black_pieces: fen_info.black_pieces,
            piece_on_square: fen_info.piece_on_square,
        };

        board.update_attack_bitboards();
        board.create_zobrist_key();
        Ok(board)
    }

    pub fn is_legal_move(&self, movement: &Move) -> bool {
        // This move was received from the user, check that it is indeed legal
        // We do this by making sure it exists in the list of allowed moves
        // Even though we generate all the moves just to check, this is only
        // done for user-provided moves. The moves made by the engine when
        // it is analyzing a position bypass this check
        self.legal_moves().contains(movement)
    }

    pub fn get_pieces_squares(&self) -> &[Option<PieceType>; 64] {
        &self.piece_on_square
    }

    // Make a given move and return a new move, **assuming that the move is legal**
    // Moves provided by the user should always be checked using .is_legal_move() first
    pub fn make_move(&mut self, movement: &Move) {
        // Initialize the move undo data that we will store
        let mut move_undo_data = MoveUndoData {
            state: self.state,
            ep: false,
            captured_piece: None,
        };

        // If there is one active e.p. square, remove it from the zobrist key
        // The e.p. flag for the zobrist key must only be flipped if the
        // side to move has a pawn ready to capture the e.p. square, which
        // is the condition that sets in in the first place
        // This is done before update_en_passant() and outside it because
        // that function clears the e.p. square, and we still need it for
        // the move_piece() method, so we can know if a capture is e.p. or not
        if self.update_ep_zobrist(self.turn_color()) {
            self.state.zobrist_key ^= zobrist::get_key_ep_square(self.ep_square().first_piece_index());
        }

        // Perform the movement in question
        if matches!(movement, Move::LongCastle | Move::ShortCastle) {
            self.castle(movement, &mut move_undo_data);
            // Castling calls move_piece twice, so the half-turn counter for
            // the 50 move rule is updated twice, that's why we must substract 1
            self.state.fifty_move_rule_counter -= 1;
        } else {
            self.move_piece(movement, &mut move_undo_data);
        }

        // Update the en passant data
        self.update_en_passant(movement);

        // Update the current color to play and the number of total turns,
        // if black just moved
        self.turn = !self.turn;
        self.state.zobrist_key ^= zobrist::get_key_white_turn();
        self.full_turns += self.turn.to_index() as u16; // 0 if black, 1 if white

        self.update_attack_bitboards();
        self.plies += 1;

        // Store the move data to undo it later
        self.previous_moves.push(move_undo_data);
    }

    // Unmakes a move. It is very important that the provided move is the
    // last one that was played, otherwise the board will be in an inconsistent state.
    // It is assumed that there is at least one move to undo. This function should
    // never be called by an user.
    pub fn unmake_move(&mut self, movement: &Move) {
        // Recover the aux info about the move that we have to undo, which
        // will be the latest one in the stack
        let move_undo_data = self.previous_moves.pop().unwrap();

        // Restore the previous state and roll back some counters
        self.state = move_undo_data.state;
        self.full_turns -= self.turn.to_index() as u16; // 0 if black, 1 if white
        self.plies -= 1;
        self.turn = !self.turn;

        // If this is a castling move, we un-castle using a specialized function
        if movement.is_castle() {
            self.undo_castle(movement);
        } else {
            let from_bb = BitBoard::from_square(movement.from());
            let to_bb = BitBoard::from_square(movement.to());
            let (moving_color, enemy_color) = (self.turn_color(), !self.turn_color());

            // Undo the piece's movement
            * self.get_color_bitboard_mut(moving_color) ^= from_bb | to_bb;
            self.all_pieces ^= from_bb | to_bb;
            let piece_moving = self.piece_on(movement.to()).unwrap();
            self.piece_on_square[movement.to() as usize] = None;

            *self.get_pieces_mut(moving_color).get_pieces_of_type_mut(piece_moving) ^= to_bb;
            // The type of piece to add is the same one that is moving, 
            // except if this is a pawn promotion
            let piece_before = if movement.is_promotion() {Pawn} else {piece_moving};
            *self.get_pieces_mut(moving_color).get_pieces_of_type_mut(piece_before) ^= from_bb;
            self.piece_on_square[movement.from() as usize] = Some(piece_before);

            // If a piece was captured, restore it
            // If it was an en passant capture, we must restore it at the current e.p. target
            if move_undo_data.ep {
                let ep_square = self.ep_square();
                let target_bb = if self.turn_color() == White {ep_square >> 8} else {ep_square << 8};
                *self.get_color_bitboard_mut(enemy_color) |= target_bb;
                self.all_pieces |= target_bb;
                *self.get_pieces_mut(enemy_color).get_pieces_of_type_mut(Pawn) |= target_bb;
                self.piece_on_square[target_bb.first_piece_index() as usize] = Some(Pawn);
            } 
            // Otherwise, it was just a normal capture, restore it in the "to" square of the move
            else if let Some(captured_type) = move_undo_data.captured_piece {
                *self.get_color_bitboard_mut(enemy_color) |= to_bb;
                self.all_pieces |= to_bb;
                *self.get_pieces_mut(enemy_color).get_pieces_of_type_mut(captured_type) |= to_bb;
                self.piece_on_square[movement.to() as usize] = move_undo_data.captured_piece;
            }
        }
    }

    // Performs a null move, which means passing
    pub fn make_null_move(&mut self) {
        // Push the current state onto the stack
        let move_data = MoveUndoData {
            ep: false,
            captured_piece: None,
            state: self.state
        };
        self.previous_moves.push(move_data);

        // If there is an active e.p. square, remove it
        if self.update_ep_zobrist(self.turn_color()) {
            self.state.zobrist_key ^= zobrist::get_key_ep_square(self.ep_square().first_piece_index());
        }
        self.state.en_passant_target.clear();

        // Update the turn, the ply count and the full move count
        self.turn = !self.turn;
        self.state.zobrist_key ^= zobrist::get_key_white_turn();
        self.plies += 1;
        self.full_turns += self.turn.to_index() as u16; // 0 if black, 1 if white
    }

    // Unmakes a null move. Similarly to unmake_move(), it is very important that
    // this function is only called when the most recent move is a null move.
    pub fn unmake_null_move(&mut self) {
        let move_undo_data = self.previous_moves.pop().unwrap();
        self.state = move_undo_data.state;
        self.full_turns -= self.turn.to_index() as u16; // 0 if black, 1 if white
        self.plies -= 1;
        self.turn = !self.turn;
    }

    pub fn pseudolegal_moves(&self) -> Vec<Move> {
        if self.is_draw() {
            vec![]
        } else {
            movegen::get_pseudolegal_moves(self, self.turn_color())
        }
    }

    pub fn pseudolegal_caps(&self) -> Vec<Move> {
        if self.is_draw() {
            vec![]
        } else {
            movegen::get_pseudolegal_caps_proms(self)
        }
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        let mut board = self.clone(); // TO-DO: refactor this as soon as we have better legality testing, its kinda shit
        board
            .pseudolegal_moves().into_iter()
            .filter(|mv| matches!(mv, Move::ShortCastle | Move::LongCastle) ||
                         {
                            board.make_move(mv);
                            let is_check = board.is_check(!board.turn_color());
                            board.unmake_move(mv);
                            !is_check
                         }
            )
            .collect()
    }

    pub fn is_check(&self, color: Color) -> bool {
        match color {
            White => (self.white_pieces.king & self.state.black_attacks).is_not_empty(),
            Black => (self.black_pieces.king & self.state.white_attacks).is_not_empty()
        }
    }

    // A position is a draw by insufficient material if both sides have either
    // only K, KB or KN
    pub fn is_draw_by_material(&self) -> bool {
        // Return false if the current position is a check, since otherwise
        // we would return an empty list of available moves in a position that is
        // a check, which would be interpreted as a checkmate
        let is_check = self.is_check(self.turn_color());

        let n_whites = self.all_whites.count();
        let n_blacks = self.all_blacks.count();

        !is_check && (n_whites == 1 || n_whites == 2 && (self.white_pieces.bishops.count() == 1 || self.white_pieces.knights.count() == 1)) 
                  && (n_blacks == 1 || n_blacks == 2 && (self.black_pieces.bishops.count() == 1 || self.black_pieces.knights.count() == 1)) 
    }

    // Returns whether the current position only has pawns, or if it has
    // 7 pieces or less. This is done to prevent null moves in the
    // endgame, which may misevaluate zugzwang positions.
    pub fn only_pawns_or_endgame(&self) -> bool {
        let piece_count = self.all_pieces.count();
        piece_count <= 7 || self.only_pawns()
    }

    // Returns whether the current position only has pawns and kings
    pub fn only_pawns(&self) -> bool {
        let piece_count = self.all_pieces.count();
        piece_count == self.black_pieces.pawns.count() + self.white_pieces.pawns.count() + 2
    }

    pub fn ep_square(&self) -> BitBoard {
        self.state.en_passant_target
    }

    pub fn castling_info(&self) -> &CastlingRights {
        &self.state.castling_rights
    }

    pub fn turn_number(&self) -> u16 {
        self.full_turns
    }

    pub fn turn_color(&self) -> Color {
        self.turn
    }

    pub fn zobrist_key(&self) -> u64 {
        self.state.zobrist_key
    }

    pub fn current_ply(&self) -> u16 {
        self.plies
    }

    pub fn fifty_move_rule_counter(&self) -> u16 {
        self.state.fifty_move_rule_counter
    }

    pub fn get_pieces(&self, color: Color) -> &Pieces {
        match color {
            White => &self.white_pieces,
            Black => &self.black_pieces
        }
    }

    pub fn piece_on(&self, square: u8) -> &Option<PieceType> {
        &self.piece_on_square[square as usize]
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

    pub fn get_attack_bitboard(&self, color: Color) -> BitBoard {
        match color {
            White => self.state.white_attacks,
            Black => self.state.black_attacks,
        }
    }

    pub fn last_moved(&self) -> u8 {
        self.state.last_moved
    }

    pub fn perft(&mut self, depth: usize) -> u64 {
        self._perft(depth, true)
    }

    ///////////////////////////////////////////////////////////////////////////
    /// Private auxiliary functions
    
    fn move_piece(&mut self, movement: &Move, move_undo_data: &mut MoveUndoData) {
        // This function is called with legal moves, so we can assume
        // that the piece exists in the "from" position and can move to the
        // target position. It only does single moves, not castling.
        // It is also assumed that it is only called by normal or promotion
        // moves, which implement .from() and .to()
        let from_bb = BitBoard::from_square(movement.from());
        let to_bb = BitBoard::from_square(movement.to());
        let (moving_color, enemy_color) = (self.turn_color(), !self.turn_color());
        let piece_moving = self.piece_on(movement.from()).unwrap();

        let enemy_pieces_bb = self.get_color_bitboard(enemy_color);
        let mut captured_piece = None;

        // If there is a piece in the destination square, remove it

        // First check for e.p., where the square we must remove is different
        // A movement is an en passant capture if it's a pawn moving and its destination
        // is the active en passant square. Note that it is impossible for any pawn moves
        // to the e.p. square not to be an en passant capture, since a pawn cannot
        // move forward to it since, by definition, the "from" square would be
        // occupied by the pawn that caused the e.p. square to become active in
        // the first place.
        if piece_moving == Pawn && to_bb == self.ep_square() {
            move_undo_data.ep = true; // Set the e.p. flag on the undo data
            let target_ep = match moving_color {
                White => movement.to() - 8,
                Black => movement.to() + 8,
            };

            // Remove the pawn that was captured e.p.
            let target_bb = BitBoard::from_square(target_ep);
            *self.get_pieces_mut(enemy_color).get_pieces_of_type_mut(Pawn) ^= target_bb;
            *self.get_color_bitboard_mut(enemy_color) ^= target_bb;
            self.all_pieces ^= target_bb;
            *self.piece_on_mut(target_ep) = None;
        
            // The type of the captured piece is not really needed here, since it's always a pawn
            captured_piece = Some(Pawn);
            // Update the zobrist key removing the captured pawn
            self.state.zobrist_key ^= zobrist::get_key_for_piece(Pawn, enemy_color, target_ep);
            
        // Not an en-passant, just a normal capture
        } else if (enemy_pieces_bb & to_bb).is_not_empty() {
            self.get_pieces_mut(enemy_color).remove_in_all(to_bb); // TO-DO: optimizable?
            *self.get_color_bitboard_mut(enemy_color) ^= to_bb;
            captured_piece = *self.piece_on(movement.to());
            // Update the zobrist key (no need to update piece_on_square since it'll be overwritten)
            self.state.zobrist_key ^= zobrist::get_key_for_piece(captured_piece.unwrap(), enemy_color, movement.to());
        }

        // Move the piece, depending on whether this is a pawn promotion or not
        self.state.zobrist_key ^= zobrist::get_key_for_piece(piece_moving, moving_color, movement.from());
        *self.piece_on_mut(movement.from()) = None;

        // Update bitboards: the bitboard for all pieces of that color always
        // reflects the movement
        *self.get_color_bitboard_mut(moving_color) ^= from_bb | to_bb;
        // The bitboard for all pieces has the origin square removed...
        self.all_pieces ^= from_bb;
        // ...and the target square is set if it wasn't already
        self.all_pieces |= to_bb;

        let our_pieces = self.get_pieces_mut(moving_color);
        if let Move::PawnPromotion { promote_to, ..} = movement {
            *our_pieces.get_pieces_of_type_mut(Pawn) ^= from_bb;
            *our_pieces.get_pieces_of_type_mut(*promote_to) ^= to_bb;
            self.state.zobrist_key ^= zobrist::get_key_for_piece(*promote_to, moving_color, movement.to());
            *self.piece_on_mut(movement.to()) = Some(*promote_to);
        } else {
            *our_pieces.get_pieces_of_type_mut(piece_moving) ^= from_bb | to_bb;
            self.state.zobrist_key ^= zobrist::get_key_for_piece(piece_moving, moving_color, movement.to());
            *self.piece_on_mut(movement.to()) = Some(piece_moving);
        }

        // Update the counter towards the 50 move rule
        if captured_piece.is_some() || piece_moving == Pawn {
            self.state.fifty_move_rule_counter = 0;
        } else {
            self.state.fifty_move_rule_counter += 1;
        }

        // Update castling rights
        self.state.zobrist_key ^= zobrist::get_key_castling(self.castling_info());
        self.update_castling_rights(movement);
        self.state.zobrist_key ^= zobrist::get_key_castling(self.castling_info());

        // Update the last moved piece
        self.state.last_moved = movement.to();

        // Copy the captured piece to the move undo data
        move_undo_data.captured_piece = captured_piece;
    }

    fn castle(&mut self, movement: &Move, move_undo_data: &mut MoveUndoData) {
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

        let king_move = Move::Normal { from: king_from, to: king_to };
        let rook_move = Move::Normal { from: rook_from, to: rook_to };

        self.move_piece(&king_move, move_undo_data);
        self.move_piece(&rook_move, move_undo_data);
    }

    // This move undos a castle, assuming that the castling move was legal
    fn undo_castle(&mut self, movement: &Move) {
        let color = self.turn_color();
        let short = matches!(movement, Move::ShortCastle);

        let row_start = if color == White { 0 } else { 56 };
        
        // From and to are reversed w.r.t. the castle() method
        let (king_to, king_from, rook_to, rook_from) = if short {
            (row_start + 3, row_start + 1, row_start, row_start + 2)
        } else {
            (row_start + 3, row_start + 5, row_start + 7, row_start + 4)
        };

        // Masks to apply to the bitboards
        let (king_mask, rook_mask) = match (short, color) {
            (true, White) => (movegen::WHITE_KING_SHORT_CASTLE, movegen::WHITE_ROOK_SHORT_CASTLE),
            (false, White) => (movegen::WHITE_KING_LONG_CASTLE, movegen::WHITE_ROOK_LONG_CASTLE),
            (true, Black) => (movegen::BLACK_KING_SHORT_CASTLE, movegen::BLACK_ROOK_SHORT_CASTLE),
            (false, Black) => (movegen::BLACK_KING_LONG_CASTLE, movegen::BLACK_ROOK_LONG_CASTLE),
        };

        // Update the piece-square array
        self.piece_on_square[king_from] = None;
        self.piece_on_square[king_to] = Some(King);
        self.piece_on_square[rook_from] = None;
        self.piece_on_square[rook_to] = Some(Rook);

        // Update the bitboards
        self.all_pieces ^= king_mask | rook_mask;
        *self.get_color_bitboard_mut(color) ^= king_mask | rook_mask;
        let pieces = self.get_pieces_mut(color);
        pieces.king ^= king_mask;
        pieces.rooks ^= rook_mask;
    }

    fn update_en_passant(&mut self, movement: &Move) {
        // Remove the e.p. square
        self.state.en_passant_target.clear();

        // If this is a pawn move, check if it's a double push to set the e.p. square
        // Note: this runs *after* the piece has been moved, so the piece we are
        // looking for is in the "to" position
        if let Move::Normal {from, to} = movement {
            if self.piece_on(movement.to()) == &Some(Pawn) {
                let color = self.turn_color();
                // This is done *before* the color is updated, hence,
                // the current turn is the one that played the move
                // Pawns move in increments (white) or decrements (black) of
                // 8, so we can use that to detect if it's a double push
                if color == White && to - from == 16 {
                    self.state.en_passant_target = BitBoard::from_square(*from + 8);
                    if self.update_ep_zobrist(Black) {
                        self.state.zobrist_key ^= zobrist::get_key_ep_square(*from + 8);
                    }
                } else if color == Black && from - to == 16 {
                    self.state.en_passant_target = BitBoard::from_square(*from - 8);
                    if self.update_ep_zobrist(White) {
                        self.state.zobrist_key ^= zobrist::get_key_ep_square(*from - 8);
                    }
                }
            }
        }
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

        if self.state.castling_rights.can_castle_queenside(op_color) && to == rook_positions.1.0 {
            self.state.castling_rights.update_queenside(op_color, false);
        } else if self.state.castling_rights.can_castle_kingside(op_color) && to == rook_positions.1.1 {
            self.state.castling_rights.update_kingside(op_color, false);
        }

        // Check if we are moving our own king or one of our rooks
        // Note: this runs after the piece has been moved, so the piece we are
        // looking for is in the "to" position
        if self.piece_on(movement.to()) == &Some(King) {
            self.state.castling_rights.disable_all(color);
        } else if self.state.castling_rights.can_castle_queenside(color) && from == rook_positions.0.0 {
            self.state.castling_rights.update_queenside(color, false);
        } else if self.state.castling_rights.can_castle_kingside(color) && from == rook_positions.0.1 {
            self.state.castling_rights.update_kingside(color, false);
        }
    }

    fn get_color_bitboard_mut(&mut self, color: Color) -> &mut BitBoard {
        match color {
            White => &mut self.all_whites,
            Black => &mut self.all_blacks,
        }
    }

    fn update_attack_bitboards(&mut self) {
        self.state.white_attacks = movegen::get_controlled_squares(self, White);
        self.state.black_attacks = movegen::get_controlled_squares(self, Black);
    }

    fn create_zobrist_key(&mut self) {
        // Creates the zobrist key for this board from scratch, assuming that
        // the initial value is 0. This should only be called once, when
        // the board is created. Incremental updates are done by the corresponding
        // move-related methods.

        // First, the pieces
        for color in [Black, White] {
            for piece_type in [King, Queen, Bishop, Knight, Rook, Pawn] {
                self.get_pieces(color).get_pieces_of_type(piece_type)
                    .piece_indices()
                    .for_each(|sq| self.state.zobrist_key ^= zobrist::get_key_for_piece(piece_type, color, sq));
            }
        }

        // Then, castling rights
        self.state.zobrist_key ^= zobrist::get_key_castling(self.castling_info());

        // e.p. square, if it's set and there is a pawn ready to capture it...
        if self.update_ep_zobrist(self.turn_color()) {
            self.state.zobrist_key ^= zobrist::get_key_ep_square(self.ep_square().first_piece_index())
        }

        //...finally, white's turn
        if self.turn_color() == White {
            self.state.zobrist_key ^= zobrist::get_key_white_turn();
        }
    }

    fn get_pieces_mut(&mut self, color: Color) -> &mut Pieces {
        match color {
            White => &mut self.white_pieces,
            Black => &mut self.black_pieces
        }
    }

    fn piece_on_mut(&mut self, square: u8) -> &mut Option<PieceType> {
        &mut self.piece_on_square[square as usize]
    }

    fn update_ep_zobrist(&self, color_capturing: Color) -> bool {
        // Returns whether the current board should have the zobrist
        // flag for an active e.p. square on. This is only true if
        // the e.p. square is set, AND there is a pawn of the opposite
        // color ready to capture it.
        !self.ep_square().is_empty() && !(
            EP_ATTACKS[self.ep_square().first_piece_index() as usize] &
            self.get_pieces(color_capturing).pawns
        ).is_empty()
    }

    fn is_draw(&self) -> bool {
        self.fifty_move_rule_counter() >= 100 || self.is_draw_by_material()
    }

    fn _perft(&mut self, depth: usize, multithread: bool) -> u64 {
        if depth == 1 {
            return self.legal_moves().len() as u64;
        }

        let pseudo_moves = self.pseudolegal_moves();

       if multithread {
            pseudo_moves.into_par_iter().filter_map(|mv| {
                let mut b = self.clone();
                b.make_move(&mv);
                let res = if matches!(mv, Move::LongCastle | Move::ShortCastle) || !b.is_check(!b.turn_color()) {
                    Some(b._perft(depth - 1, false))
                } else {
                    None
                };

                b.unmake_move(&mv);
                res
            }).sum()
        } else {
            pseudo_moves.into_iter().filter_map(|mv| {
                self.make_move(&mv);
                let res = if matches!(mv, Move::LongCastle | Move::ShortCastle) || !self.is_check(!self.turn_color()) {
                    Some(self._perft(depth - 1, false))
                } else {
                    None
                };

                self.unmake_move(&mv);
                res
            }).sum()
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

    pub fn remove_in_all(&mut self, target: BitBoard) {
        let mask = !target;
        self.pawns &= mask;
        self.knights &= mask;
        self.bishops &= mask;
        self.rooks &= mask;
        self.queens &= mask;
        self.king &= mask;
    }

    pub fn combine(&self) -> BitBoard {
        self.pawns | self.knights | self.bishops | self.rooks | self.queens | self.king
    }
}
