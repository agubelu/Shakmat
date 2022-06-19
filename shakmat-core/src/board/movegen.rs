use crate::board::{Board, BitBoard};
use crate::game_elements::{Color, Color::*, PieceType::*, Move};
use crate::magic;

use super::Pieces;

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

// Bitboards with the from and to positions for the kings and rooks for castling
pub const WHITE_KING_SHORT_CASTLE: BitBoard = BitBoard::new(0x000000000000000A);
pub const WHITE_ROOK_SHORT_CASTLE: BitBoard = BitBoard::new(0x0000000000000005);
pub const WHITE_KING_LONG_CASTLE: BitBoard = BitBoard::new(0x0000000000000028);
pub const WHITE_ROOK_LONG_CASTLE: BitBoard = BitBoard::new(0x0000000000000090);
pub const BLACK_KING_SHORT_CASTLE: BitBoard = BitBoard::new(0x0A00000000000000);
pub const BLACK_ROOK_SHORT_CASTLE: BitBoard = BitBoard::new(0x0500000000000000);
pub const BLACK_KING_LONG_CASTLE: BitBoard = BitBoard::new(0x2800000000000000);
pub const BLACK_ROOK_LONG_CASTLE: BitBoard = BitBoard::new(0x9000000000000000);

// Some useful masks for pawn movements
const THIRD_RANK_MASK: BitBoard = BitBoard::new(0x0000000000FF0000);
const SIXTH_RANK_MASK: BitBoard = BitBoard::new(0x0000FF0000000000);
const WHITE_PROMOTION_RANK: BitBoard = BitBoard::new(0xFF00000000000000);
const BLACK_PROMOTION_RANK: BitBoard = BitBoard::new(0x00000000000000FF);

// Generates all pseudolegal moves
pub fn get_pseudolegal_moves(board: &Board, color: Color) -> Vec<Move> {
    let pieces = board.get_pieces(color);
    let enemy_pieces = board.get_color_bitboard(!color);
    let friendly_pieces_mask = !board.get_color_bitboard(color);
    let all_pieces = board.get_all_bitboard();

    let mut moves = generate_normal_moves(pieces, all_pieces, friendly_pieces_mask);

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

    // Finally, pawns. The funniest of pieces.
    // We need an aux vec to later transform the moves that end up in the
    // last rank to promotion moves
    let mut pawn_moves = Vec::with_capacity(50);
    let ep_square = board.ep_square();
    pieces.pawns.piece_indices().for_each(|from| {
        // Captures, which must target either an enemy piece or the e.p. square
        let cap_bb = magic::pawn_attacks(from as usize, color) & (enemy_pieces | ep_square);
        pawn_moves.extend(cap_bb.piece_indices().map(|to| Move::Normal { from, to }));

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

        pawn_moves.extend(push_bb.piece_indices().map(|to| Move::Normal { from, to }));
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

    moves
}

// Generates pseudolegal captures and promotions only
pub fn get_pseudolegal_caps_proms(board: &Board) -> Vec<Move> {
    let color = board.turn_color();
    let pieces = board.get_pieces(color);
    let enemy_pieces = board.get_color_bitboard(!color);
    let all_pieces = board.get_all_bitboard();

    // Generate only captures by providing the location of enemy pieces
    // as a mask
    let mut moves = generate_normal_moves(pieces, all_pieces, enemy_pieces);

    // Generate capturing pawn moves and promotions
    let prom_rank = match color {
        White => WHITE_PROMOTION_RANK,
        Black => BLACK_PROMOTION_RANK,
    };

    let mut pawn_moves = Vec::with_capacity(50);

    pieces.pawns.piece_indices().for_each(|from| {
        // Captures, which must target either an enemy piece or the e.p. square
        let cap_bb = magic::pawn_attacks(from as usize, color) & (enemy_pieces | board.ep_square());
        pawn_moves.extend(cap_bb.piece_indices().map(|to| Move::Normal { from, to }));

        // Pushes that end up in the promotion rank
        let push_bb = magic::pawn_pushes(from as usize, color) & !all_pieces & prom_rank;
        pawn_moves.extend(push_bb.piece_indices().map(|to| Move::Normal { from, to }));
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

// Generates moves for all pieces except pawns and castling moves using a mask
// This filter will be the inverse of the position of friendly pieces in normal
// move generation (to avoid capturing them), or the position of enemy pieces
// when generating only capture moves
fn generate_normal_moves(pieces: &Pieces, all_pieces: BitBoard, mask: BitBoard) -> Vec<Move> {
    // Queen
    let queen_moves = pieces.queens.piece_indices().flat_map(|from| {
        let move_bb = magic::queen_moves(from as usize, all_pieces) & mask;
        move_bb.piece_indices().map(move |to| Move::Normal { from, to })
    });

    // Rook
    let rook_moves = pieces.rooks.piece_indices().flat_map(|from| {
        let move_bb = magic::rook_moves(from as usize, all_pieces) & mask;
        move_bb.piece_indices().map(move |to| Move::Normal { from, to })
    });

    // Bishop
    let bishop_moves = pieces.bishops.piece_indices().flat_map(|from| {
        let move_bb = magic::bishop_moves(from as usize, all_pieces) & mask;
        move_bb.piece_indices().map(move |to| Move::Normal { from, to })
    });

    // Horsey
    let knight_moves = pieces.knights.piece_indices().flat_map(|from| {
        let move_bb = magic::knight_moves(from as usize) & mask;
        move_bb.piece_indices().map(move |to| Move::Normal { from, to })
    });

    // King
    let king_moves = pieces.king.piece_indices().flat_map(|from| {
        let move_bb = magic::king_moves(from as usize) & mask;
        move_bb.piece_indices().map(move |to| Move::Normal { from, to })
    });

    queen_moves.chain(bishop_moves).chain(rook_moves)
        .chain(knight_moves).chain(king_moves).collect()
}

fn in_promotion_rank(pos: u8, color: Color) -> bool {
    match color {
        Color::Black => pos < 8,
        Color::White => pos > 55
    }
}