use shakmat_core::{Board, Move, PieceType};

// Heuristic values of different kinds of moves
const TT_MOVE: i16 = 10_000; // Best move stored in the transposition table for this depth
const LAST_RECAPTURE: i16 = 1000; // Capture of the last moved piece
const KILLER: i16 = 50; // Killer move
const CAPTURE: i16 = 100; // Normal captures: 100 + MVV-LVA value (see below)

// Struct to hold a pair of (Move, move heuristical value)
pub struct RatedMove {
    pub mv: Move,
    pub score: i16
}

// Receives the pseudolegal moves for the current position and, optionally,
// the best move according to the transposition table
// Returns a list of RatedMoves according to the heuristics above.
pub fn order_moves(moves: Vec<Move>, board: &Board, tt_move: Option<Move>, killers: &[Move]) -> Vec<RatedMove> {
    let mut rated_moves: Vec<RatedMove> = moves.into_iter().map(|mv| rate_move(mv, tt_move, board, killers)).collect();
    rated_moves.sort_unstable_by_key(|rm| rm.score);
    rated_moves
}

// Takes a move by value and returns a struct with that move
// and its heuristic value. PV moves are rated the highest, then captures
fn rate_move(mv: Move, pv_move: Option<Move>, board: &Board, killers: &[Move]) -> RatedMove {
    let score = if pv_move == Some(mv) {
        TT_MOVE
    } else if let Some(captured) = mv.piece_captured(board) {
        CAPTURE + value_of_capture(captured) - value_of_attacker(mv.piece_moving(board))
    } else if matches!(mv, Move::Normal{to, ..} | Move::PawnPromotion{to, ..} if to == board.last_moved()) {
        // Note: the "if" applies to both patterns, not just the PawnPromotion move
        LAST_RECAPTURE
    } else if killers[0] == mv || killers[1] == mv {
        KILLER
    } else {
        0
    };

    // The move rating is negated so that higher rated moves go first
    RatedMove { mv, score: -score }
}

// Tables for Most Valuable Victim - Least Valuable Aggressor (MVV-LVA)
// Attempts to provide a heuristic for capturing moves by
// capturing with the least valuable piece
const fn value_of_attacker(piece: PieceType) -> i16 {
    match piece {
        PieceType::Pawn => 10,
        PieceType::Knight => 30,
        PieceType::Bishop => 30,
        PieceType::Rook => 50,
        PieceType::Queen => 90,
        PieceType::King => 99,
    }
}

const fn value_of_capture(piece: PieceType) -> i16 {
    match piece {
        PieceType::Pawn => 100,
        PieceType::Knight => 300,
        PieceType::Bishop => 300,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::King => 9999, // Doesn't happen since the king is never captured
    }
}
