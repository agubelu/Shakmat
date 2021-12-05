use std::ops::BitAnd;

use crate::chess::{Move, Board, Color, PieceType, BitBoard};
use Color::*;
use PieceType::*;

use crate::chess::magic;

// Bitboards that have 1's in the required spaces to castle for
// both colors
const WHITE_SHORT_CASTLE_BB: BitBoard = BitBoard::new(6);
const WHITE_LONG_CASTLE_BB: BitBoard = BitBoard::new(112);
const BLACK_SHORT_CASTLE_BB: BitBoard = BitBoard::new(0x0600000000000000);
const BLACK_LONG_CASTLE_BB: BitBoard = BitBoard::new(0x7000000000000000);

const THIRD_RANK_MASK: BitBoard = BitBoard::new(0x0000000000FF0000);
const SIXTH_RANK_MASK: BitBoard = BitBoard::new(0x0000FF0000000000);

pub fn get_pseudolegal_moves(board: &Board, color: Color) -> Vec<Move> {
    let mut moves = Vec::with_capacity(100); // just to be safe and avoid reallocations
    let pieces = board.get_pieces(color);
    let enemy_pieces = board.get_color_bitboard(!color);
    let friendly_pieces_mask = !board.get_color_bitboard(color);
    let all_pieces = board.get_all_bitboard();

    // Ah yes, pawns. The funniest of pieces.
    // We need an aux vec to later transform the moves that end up in the
    // last rank to promotion moves
    let mut pawn_moves = Vec::with_capacity(50);
    let ep_square = board.ep_square();
    let ep_index = ep_square.get_u64().trailing_zeros() as u8;
    pieces.pawns.piece_indices().for_each(|from| {
        // Captures, which must target either an enemy piece or the e.p. square
        let cap_bb = magic::pawn_attacks(from as usize, color) & (enemy_pieces | ep_square);
        pawn_moves.extend(cap_bb.piece_indices().map(|to| Move::Normal { from, to, piece: Pawn, ep: to == ep_index }));

        // Next, pushes. Going one step forward is always an option, if nothing is
        // in the way
        let mut push_bb = magic::pawn_pushes(from as usize, color) & !all_pieces;

        // If it's a white pawn in the second rank, disable the double push if there
        // is a piece in front of it
        if color == Color::White && from < 16 {
            push_bb &= !((all_pieces & THIRD_RANK_MASK) << 8);
        } else if color == Color::Black && from > 47 {
            push_bb &= !((all_pieces & SIXTH_RANK_MASK) >> 8);
        }

        pawn_moves.extend(push_bb.piece_indices().map(|to| Move::Normal { from, to, piece: Pawn, ep: false }));
    });

    // Transform the pawn moves into promotions if needed
    moves.extend(pawn_moves.into_iter().flat_map(|mv| {
        if in_promotion_rank(mv.to(), color) {
            vec![
                Move::PawnPromotion { from: mv.from(), to: mv.to(), promote_to: Queen },
                Move::PawnPromotion { from: mv.from(), to: mv.to(), promote_to: Rook },
                Move::PawnPromotion { from: mv.from(), to: mv.to(), promote_to: Bishop },
                Move::PawnPromotion { from: mv.from(), to: mv.to(), promote_to: Knight }
            ].into_iter()
        } else {
            vec![mv].into_iter()
        }
    }));

    // Rook
    pieces.rooks.piece_indices().for_each(|from| {
        let move_bb = magic::rook_moves(from as usize, all_pieces) & friendly_pieces_mask;
        moves.extend(move_bb.piece_indices().map(|to| Move::Normal { from, to, piece: Rook, ep: false }));
    });

    // Bishop
    pieces.bishops.piece_indices().for_each(|from| {
        let move_bb = magic::bishop_moves(from as usize, all_pieces) & friendly_pieces_mask;
        moves.extend(move_bb.piece_indices().map(|to| Move::Normal { from, to, piece: Bishop, ep: false }));
    });

    // Queen
    pieces.queens.piece_indices().for_each(|from| {
        let move_bb = magic::queen_moves(from as usize, all_pieces) & friendly_pieces_mask;
        moves.extend(move_bb.piece_indices().map(|to| Move::Normal { from, to, piece: Queen, ep: false }));
    });

    // Horsey
    pieces.knights.piece_indices().for_each(|from| {
        let move_bb = magic::knight_moves(from as usize) & friendly_pieces_mask;
        moves.extend(move_bb.piece_indices().map(|to| Move::Normal { from, to, piece: Knight, ep: false }));
    });

    // King
    // First, the simple 1-square moves
    pieces.king.piece_indices().for_each(|from| {
        let move_bb = magic::king_moves(from as usize) & friendly_pieces_mask;
        moves.extend(move_bb.piece_indices().map(|to| Move::Normal { from, to, piece: King, ep: false }));
    });

    // Next, castling. Here we only test that the needed squares are
    // empty and that the king has the right to castle, we test for checks
    // later on upon legal move filtering
    // We can assume that, if the color has the right to castle, both
    // the king and the rook are in the required positions
    let (short_bb, long_bb) = match color {
        White => (WHITE_SHORT_CASTLE_BB, WHITE_LONG_CASTLE_BB),
        Black => (BLACK_SHORT_CASTLE_BB, BLACK_LONG_CASTLE_BB)
    };

    if board.castling_info().can_castle_kingside(color) && (all_pieces & short_bb).is_empty() {
        moves.push(Move::ShortCastle);
    }

    if board.castling_info().can_castle_queenside(color) && (all_pieces & long_bb).is_empty() {
        moves.push(Move::LongCastle);
    }

    moves
}

pub fn get_controlled_squares(board: &Board, color: Color) -> BitBoard {
    // TODO idea: map and reduce by or'ing, and OR controlled with the result?
    let mut controlled = BitBoard::new(0);
    let our_pieces = board.get_pieces(color);
    let all_pieces = board.get_all_bitboard();

    our_pieces.king.piece_indices().for_each(|from| controlled |= magic::king_moves(from as usize));
    our_pieces.knights.piece_indices().for_each(|from| controlled |= magic::knight_moves(from as usize));
    our_pieces.queens.piece_indices().for_each(|from| controlled |= magic::queen_moves(from as usize, all_pieces));
    our_pieces.bishops.piece_indices().for_each(|from| controlled |= magic::bishop_moves(from as usize, all_pieces));
    our_pieces.rooks.piece_indices().for_each(|from| controlled |= magic::rook_moves(from as usize, all_pieces));
    our_pieces.pawns.piece_indices().for_each(|from| controlled |= magic::pawn_attacks(from as usize, color));

    controlled
}

fn in_promotion_rank(pos: u8, color: Color) -> bool {
    match color {
        Color::Black => pos < 8,
        Color::White => pos > 55
    }
}