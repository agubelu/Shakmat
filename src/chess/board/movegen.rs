use crate::chess::{BBMove, BBBoard, Color, PieceType, BitBoard};
use Color::*;
use PieceType::*;

use crate::chess::magic;

// Bitboards that have 1's in the required spaces to castle for
// both colors
const WHITE_SHORT_CASTLE_BB: BitBoard = BitBoard::new(6);
const WHITE_LONG_CASTLE_BB: BitBoard = BitBoard::new(112);
const BLACK_SHORT_CASTLE_BB: BitBoard = BitBoard::new(0x0600000000000000);
const BLACK_LONG_CASTLE_BB: BitBoard = BitBoard::new(0x7000000000000000);

pub fn get_pseudolegal_moves(board: &BBBoard, color: Color) -> Vec<BBMove> {
    let mut moves = Vec::with_capacity(50); // just to be safe and avoid reallocations
    let pieces = board.get_pieces(color);
    let friendly_pieces_mask = !board.get_color_bitboard(color);
    let all_pieces = board.get_all_bitboard();

    // Pawns
    // TO-DO

    // Rook
    pieces.rooks.piece_indices().for_each(|from| {
        let move_bb = magic::rook_moves(from as usize, all_pieces) & friendly_pieces_mask;
        move_bb.piece_indices().for_each(|to| moves.push( 
            BBMove::Normal { from, to, piece: Rook, ep: false } 
        ))
    });

    // Bishop
    pieces.bishops.piece_indices().for_each(|from| {
        let move_bb = magic::bishop_moves(from as usize, all_pieces) & friendly_pieces_mask;
        move_bb.piece_indices().for_each(|to| moves.push( 
            BBMove::Normal { from, to, piece: Bishop, ep: false } 
        ))
    });

    // Queen
    pieces.queens.piece_indices().for_each(|from| {
        let move_bb = magic::queen_moves(from as usize, all_pieces) & friendly_pieces_mask;
        move_bb.piece_indices().for_each(|to| moves.push( 
            BBMove::Normal { from, to, piece: Queen, ep: false } 
        ))
    });

    // Horsey
    pieces.knights.piece_indices().for_each(|from| {
        let move_bb = magic::knight_moves(from as usize) & friendly_pieces_mask;
        move_bb.piece_indices().for_each(|to| moves.push( 
            BBMove::Normal { from, to, piece: Knight, ep: false } 
        ))
    });

    // King
    // First, the simple 1-square moves
    pieces.king.piece_indices().for_each(|from| {
        let move_bb = magic::king_moves(from as usize) & friendly_pieces_mask;
        move_bb.piece_indices().for_each(|to| moves.push( 
            BBMove::Normal { from, to, piece: King, ep: false } 
        ))
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
        moves.push(BBMove::ShortCastle);
    }

    if board.castling_info().can_castle_queenside(color) && (all_pieces & long_bb).is_empty() {
        moves.push(BBMove::LongCastle);
    }

    moves
}