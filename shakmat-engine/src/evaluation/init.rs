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

            // King rings: they are defined as the squares around the king,
            // plus 3 squares in front facing the enemy side
            let file_diff = (file as isize - other_file as isize).abs();
            let rank_diff = (rank as isize - other_rank as isize).abs();
            let diff1_file = file_diff <= 1;
            let diff1_rank = rank_diff <= 1;
            let diff2_rank = rank_diff <= 2;

            // The squares around this one can be added to the king ring of
            // both colors
            if diff1_file && diff1_rank {
                unsafe {
                    masks::WHITE_KING_RING[pos] |= bb;
                    masks::BLACK_KING_RING[pos] |= bb;
                }
            }

            // If this square is in a higher rank, at most 1 file away, and
            // at most 2 ranks away, add it to white's king ring to add those
            // 3 squares facing the enemy side
            if other_rank > rank && diff1_file && diff2_rank {
                unsafe { masks::WHITE_KING_RING[pos] |= bb };
            }

            // Same for black, but it has to be a lower rank to face white's side
            if other_rank < rank && diff1_file && diff2_rank {
                unsafe { masks::BLACK_KING_RING[pos] |= bb };
            }

            // Passed pawn masks: Add if the rank is in front (white)
            // or behind (black), and the file is the same or one of the
            // sides (maximum diff of 1)
            if diff1_file && other_rank > rank {
                unsafe { masks::WHITE_PASSED_PAWN[pos] |= bb };
            }

            if diff1_file && other_rank < rank {
                unsafe { masks::BLACK_PASSED_PAWN[pos] |= bb };
            }
        }
    }
}