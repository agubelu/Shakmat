use std::fmt::{Display, Result};

use crate::chess::{Board, Color, PieceType, Position, Move, Color::*, PieceType::*};
use crate::chess::position::{UP, DOWN, LEFT, RIGHT, UP_LEFT, UP_RIGHT, DOWN_LEFT, DOWN_RIGHT};

#[derive(Debug, Clone, Copy)]
pub struct Piece {
    color: Color,
    piece_type: PieceType,
    position: Position,
}

impl Piece {
    pub fn new(color: Color, piece_type: PieceType, position: Position) -> Self {
        Piece { color, piece_type, position }
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn piece_type(&self) -> PieceType {
        self.piece_type
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn get_pseudolegal_moves(&self, board: &Board) -> Vec<Move> {
        match self.piece_type {
            King => self.get_moves_king(board),
            Knight => self.get_moves_knight(board),
            Pawn => self.get_moves_pawn(board),
            Bishop => self.get_moves_bishop(board),
            Rook => self.get_moves_rook(board),
            Queen => self.get_moves_queen(board),
        }
    }

    pub fn get_legal_moves(&self, board: &Board) -> Vec<Move> {
        self.get_pseudolegal_moves(board)
            .into_iter()
            .filter(|&m| {
                // Castling legality is checked in move generation
                matches!(m, Move::LongCastle) || matches!(m, Move::ShortCastle) ||
                !board.make_move(m, false).is_check(self.color)
            })
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
    /// it? Im not sure how that would perform in terms of efficiency, because
    /// then the board would have to contain Box<dyn Piece> instead of Piece
    fn get_moves_king(&self, board: &Board) -> Vec<Move> {
        let pos = self.position();
        let mut moves: Vec<Move> = pos.king_moves()
            .iter()
            .filter(|&future_pos| {
                let future_square = board.get_pos(future_pos);
                future_square.is_none() || future_square.unwrap().color != self.color
            })
            .map(|future_pos| Move::NormalMove { from: *pos, to: *future_pos })
            .collect();

        // To check castling, we need to know the color of the king
        // Since the king will be in the position "pos" in the board,
        // we can unwrap it safely to get its color
        let color = board.get_pos(pos).unwrap().color;

        // Check for short castling
        // If the following is true, then the king and the kingside rook have
        // not moved
        if board.castling_info().can_castle_kingside(color) {
            // If the king has the right to castle, it is in its original position
            // So we have to check that position and 2 to the right
            // They must be empty and not under attack by the opposite color
            // (except the king's initial square, which cannot be empty for obvious reasons)
            let can_castle_short = [(0, 0), (1, 0), (2, 0)].iter()
                .all(|diff| {
                    let pos_check = pos.add_delta(diff);
                    let empty = diff.0 == 0 || board.get_pos(&pos_check).is_none();
                    let under_attack = pos_check.is_attacked_by(board, !color);
                    empty && !under_attack
                });
            if can_castle_short {
                moves.push(Move::ShortCastle);
            }
        }

        // Check for long castling, same assumptions but for the
        // queenside this time
        if board.castling_info().can_castle_queenside(color) {
            // Likewise, here we can assume the king is in starting position
            // However, we must check that the two positions to the left are
            // empty and not under attack, and that the 3rd to the left is empty
            // (it can be attacked since the king wont pass through there)
            let can_castle_long = [(0, 0), (-1, 0), (-2, 0), (-3, 0)].iter()
                .all(|diff| {
                    let pos_check = pos.add_delta(diff);
                    let empty = diff.0 == 0 || board.get_pos(&pos_check).is_none();
                    let under_attack = diff.0 > -3 && pos_check.is_attacked_by(board, !color);
                    empty && !under_attack
                });

            if can_castle_long {
                moves.push(Move::LongCastle);
            }
        }

        moves
    }

    fn get_moves_knight(&self, board: &Board) -> Vec<Move> {
        let pos = self.position();
        pos.knight_jumps()
            .iter()
            .filter(|&future_pos| {
                let future_square = board.get_pos(future_pos);
                future_square.is_none() || future_square.unwrap().color != self.color
            })
            .map(|future_pos| Move::NormalMove { from: *pos, to: *future_pos })
            .collect()
    }

    fn get_moves_bishop(&self, board: &Board) -> Vec<Move> {
        let pos = self.position();
        [UP_LEFT, UP_RIGHT, DOWN_LEFT, DOWN_RIGHT].iter()
            .flat_map(|dir| pos.trace_ray(board, *dir, self.color).0)
            .map(|future_pos| Move::NormalMove { from: *pos, to: future_pos })
            .collect()
    }

    fn get_moves_rook(&self, board: &Board) -> Vec<Move> {
        let pos = self.position();
        [UP, DOWN, LEFT, RIGHT].iter()
            .flat_map(|dir| pos.trace_ray(board, *dir, self.color).0)
            .map(|future_pos| Move::NormalMove { from: *pos, to: future_pos })
            .collect()
    }

    fn get_moves_queen(&self, board: &Board) -> Vec<Move> {
        let pos = self.position();
        let mut moves = self.get_moves_bishop(board);
        moves.extend(self.get_moves_rook(board));
        moves
    }

    fn get_moves_pawn(&self, board: &Board) -> Vec<Move> {
        let pos = self.position();
        let fwd_direction = if self.color == White { UP } else { DOWN };
        let starting_rank = if self.color == White { 1 } else { 6 };
        let promotion_rank = if self.color == White { 7 } else { 0 };
        let capture_dirs = if self.color == White { [UP_LEFT, UP_RIGHT] } else { [DOWN_LEFT, DOWN_RIGHT] };

        let mut moves = Vec::with_capacity(5);

        // Check for 1 move forward
        let move_fwd = pos.add_delta(&fwd_direction);
        if board.get_pos(&move_fwd).is_none() {
            moves.push(Move::NormalMove { from: *pos, to: move_fwd });
        }

        // Check for 2 moves forward, only possible if pawn is on the starting rank
        // and there is no piece in front of it, a.k.a., we already have 1 move
        if pos.rank == starting_rank {
            let move_2fwd = move_fwd.add_delta(&fwd_direction);
            if !moves.is_empty() && board.get_pos(&move_2fwd).is_none() {
                moves.push(Move::NormalMove { from: *pos, to: move_2fwd });
            }
        }
        
        // Check for captures and en passant
        for capture_dir in capture_dirs {
            let capture_square = pos.add_delta(&capture_dir);
            // Check that the square is actually valid (i.e. not capturing to
            // the right on the rightmost file
            // and that we have something to capture, either a piece of the
            // opposite color or the target en passant square
            if capture_square.is_valid() && (
                *board.get_en_passant_target() == Some(capture_square) ||
                board.get_pos(&capture_square).is_some() && board.get_pos(&capture_square).unwrap().color != self.color
            ) {
                moves.push(Move::NormalMove { from: *pos, to: capture_square });       
            }
        }

        // Transform all moves that end up with the pawn in the promotion rank
        // into all the allowed promotion moves
        moves.into_iter().flat_map(|mv| {
            if mv.to().rank == promotion_rank {
                vec![
                    Move::PawnPromotion { from: *mv.from(), to: *mv.to(), promote_to: Queen },
                    Move::PawnPromotion { from: *mv.from(), to: *mv.to(), promote_to: Rook },
                    Move::PawnPromotion { from: *mv.from(), to: *mv.to(), promote_to: Bishop },
                    Move::PawnPromotion { from: *mv.from(), to: *mv.to(), promote_to: Knight }
                ].into_iter()
            } else {
                vec![mv].into_iter()
            }
        })
        .collect()
    }        
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result {
        writeln!(f, "{}", self.as_char())
    }
}