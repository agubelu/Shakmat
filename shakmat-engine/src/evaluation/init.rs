use super::{masks, piece_tables};
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

            // Passed pawn masks: Add if the rank is in front (white)
            // or behind (black), and the file is the same or one of the
            // sides (maximum diff of 1)
            let same_or_next_file = (file as isize - other_file as isize).abs() <= 1;
            if same_or_next_file && other_rank > rank {
                unsafe { masks::WHITE_PASSED_PAWN[pos] |= bb };
            }

            if same_or_next_file && other_rank < rank {
                unsafe { masks::BLACK_PASSED_PAWN[pos] |= bb };
            }
           
        }
    }
}