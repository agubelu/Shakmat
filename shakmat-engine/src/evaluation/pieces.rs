use shakmat_core::{BitBoard, Board};
use shakmat_core::magic::rook_moves;

const FILES: [BitBoard; 8] = [
    BitBoard::new(0x0101010101010101),
    BitBoard::new(0x0202020202020202),
    BitBoard::new(0x0404040404040404),
    BitBoard::new(0x0808080808080808),
    BitBoard::new(0x1010101010101010),
    BitBoard::new(0x2020202020202020),
    BitBoard::new(0x4040404040404040),
    BitBoard::new(0x8080808080808080),
];

// Calculates centipawn scores for the different pieces on the board

pub fn eval_rooks(bb: BitBoard, board: &Board) -> i16 {
    bb.piece_indices().into_iter()
        .map(|i| {
            let mut score = 500;
            // Bonuses:
            // Connected rooks: 15 cp
            let moves = rook_moves(i as usize, board.get_all_bitboard());
            if !(moves & bb).is_empty() {
                score += 15;
            }

            score
        })
        .sum()
}