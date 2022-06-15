use super::masks;
use shakmat_core::Square;

// Initializes several bitboard masks used in the evaluation
pub fn init_evaluation() {
    for pos in 0..64 {
        let square = Square::new(pos as u8);
        let file = square.file();
        let rank = square.rank();
        
        for other in 0..64 {
            let other_square = Square::new(other);
            let other_file = other_square.file();
            let other_rank = other_square.rank();
            let bb = other_square.as_bitboard();
            
            // File and rank masks
            if other_file == file {
                unsafe { masks::FILES[pos] |= bb };
            }

            if other_rank == rank {
                unsafe { masks::RANKS[pos] |= bb };
            }

            // King rings: the inner ring is the squares around the
            // king, while the outer one also includes knight jumps
            let file_diff = (file as isize - other_file as isize).abs();
            let rank_diff = (rank as isize - other_rank as isize).abs();

            // Inner ring
            if file_diff <= 1 && rank_diff <= 1 {
                unsafe { masks::KING_INNER_RING[pos] |= bb }
            }

            // Outer ring
            if (file_diff == 1 && rank_diff == 2) || (rank_diff == 1 && file_diff == 2) {
                unsafe { masks::KING_OUTER_RING[pos] |= bb }
            }

            // Passed pawn masks: Add if the rank is in front (white)
            // or behind (black), and the file is the same or one of the
            // sides (maximum diff of 1)
            if file_diff <= 1 && other_rank > rank {
                unsafe { masks::WHITE_PASSED_PAWN[pos] |= bb };
            }

            if file_diff <= 1 && other_rank < rank {
                unsafe { masks::BLACK_PASSED_PAWN[pos] |= bb };
            }
        }
    }
}