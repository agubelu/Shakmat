use crate::board::{Board, BitBoard};
use crate::game_elements::{Color, Color::*, PieceType::*, Move};
use crate::magic;

// Bitboards that have 1's in the required spaces to castle for
// both colors, and those that must not be in check to castle
const WHITE_SHORT_CASTLE_BB: BitBoard = BitBoard::new(6);
const WHITE_LONG_CASTLE_BB: BitBoard = BitBoard::new(112);
const BLACK_SHORT_CASTLE_BB: BitBoard = BitBoard::new(0x0600000000000000);
const BLACK_LONG_CASTLE_BB: BitBoard = BitBoard::new(0x7000000000000000);
const WHITE_SHORT_CASTLE_CHECKS: BitBoard = BitBoard::new(14);
const WHITE_LONG_CASTLE_CHECKS: BitBoard = BitBoard::new(56);
const BLACK_SHORT_CASTLE_CHECKS: BitBoard = BitBoard::new(0x0E00000000000000);
const BLACK_LONG_CASTLE_CHECKS: BitBoard = BitBoard::new(0x3800000000000000);

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

    // Next, castling. Legality check of castling is done here too
    let (short_bb, long_bb, short_checks, long_checks) = match color {
        White => (WHITE_SHORT_CASTLE_BB, WHITE_LONG_CASTLE_BB,
                  WHITE_SHORT_CASTLE_CHECKS, WHITE_LONG_CASTLE_CHECKS),
        Black => (BLACK_SHORT_CASTLE_BB, BLACK_LONG_CASTLE_BB,
                  BLACK_SHORT_CASTLE_CHECKS, BLACK_LONG_CASTLE_CHECKS),
    };

    let attackers = board.get_attack_bitboard(!color);

    if board.castling_info().can_castle_kingside(color) && (all_pieces & short_bb).is_empty()
        && (attackers & short_checks).is_empty()  {
        moves.push(Move::ShortCastle);
    }

    if board.castling_info().can_castle_queenside(color) && (all_pieces & long_bb).is_empty()
        && (attackers & long_checks).is_empty() {
        moves.push(Move::LongCastle);
    }

    moves
}

pub fn get_controlled_squares(board: &Board, color: Color) -> BitBoard {
    let mut controlled = BitBoard::new(0);
    let our_pieces = board.get_pieces(color);
    let all_pieces = board.get_all_bitboard();

    controlled |= our_pieces.king.piece_indices().map(|from| magic::king_moves(from as usize)).reduce(|a, b| a | b).unwrap_or_default();
    controlled |= our_pieces.knights.piece_indices().map(|from| magic::knight_moves(from as usize)).reduce(|a, b| a | b).unwrap_or_default();
    controlled |= our_pieces.queens.piece_indices().map(|from| magic::queen_moves(from as usize, all_pieces)).reduce(|a, b| a | b).unwrap_or_default();
    controlled |= our_pieces.bishops.piece_indices().map(|from| magic::bishop_moves(from as usize, all_pieces)).reduce(|a, b| a | b).unwrap_or_default();
    controlled |= our_pieces.rooks.piece_indices().map(|from| magic::rook_moves(from as usize, all_pieces)).reduce(|a, b| a | b).unwrap_or_default();
    controlled |= our_pieces.pawns.piece_indices().map(|from| magic::pawn_attacks(from as usize, color)).reduce(|a, b| a | b).unwrap_or_default();

    controlled
}

fn in_promotion_rank(pos: u8, color: Color) -> bool {
    match color {
        Color::Black => pos < 8,
        Color::White => pos > 55
    }
}