use shakmat_core::{Board, Move, PieceType};
use super::history::HistoryTable;

// Heuristic values of different kinds of moves
// The priorities and assigned score ranges are as follows:
// Move stored in the TT: Max
// Recapture of last moved piece: Max - 1
// Captures using MMV-LVA: [Max - 1000, Max - 1)
// Primary killer move: Max - 1001
// Secondary killer move: Max - 1002
// History heuristics: rest

pub type MoveScore = i32;

const TT_MOVE: MoveScore = MoveScore::MAX; // Best move stored in the transposition table for this depth
const LAST_RECAPTURE: MoveScore = MoveScore::MAX - 1; // Capture of the last moved piece
const CAPTURE_BASE_VAL: MoveScore = MoveScore::MAX - 1000; // Base value for any other capture
const PRIMARY_KILLER: MoveScore = CAPTURE_BASE_VAL - 1; // Primary killer move for this depth
const SECONDARY_KILLER: MoveScore = PRIMARY_KILLER - 1; // Secondary killer move for this depth
pub const MAX_HISTORY_VAL: MoveScore = SECONDARY_KILLER - 1;

// Struct to hold a pair of (Move, move heuristical value)
pub struct RatedMove {
    pub mv: Move,
    pub score: MoveScore
}

// Receives the pseudolegal moves for the current position and, optionally,
// the best move according to the transposition table
// Returns a list of RatedMoves according to the heuristics above.
pub fn order_moves(moves: Vec<Move>, board: &Board, tt_move: Option<Move>, killers: &[Move], history: &HistoryTable) -> Vec<RatedMove> {
    let mut rated_moves: Vec<RatedMove> = moves.into_iter().map(|mv| rate_move(mv, tt_move, board, killers, history)).collect();
    rated_moves.sort_unstable_by_key(|rm| rm.score);
    rated_moves
}

// Takes a move by value and returns a struct with that move
// and its heuristic value according to the consts above
fn rate_move(mv: Move, pv_move: Option<Move>, board: &Board, killers: &[Move], history: &HistoryTable) -> RatedMove {
    let score = if pv_move == Some(mv) {
        TT_MOVE
    } else if matches!(mv, Move::Normal{to, ..} | Move::PawnPromotion{to, ..} if to == board.last_moved()) {
        // Note: the "if" applies to both patterns, not just the PawnPromotion move
        LAST_RECAPTURE
    } else if let Some(captured) = mv.piece_captured(board) {
        CAPTURE_BASE_VAL + value_of_capture(captured) - value_of_attacker(mv.piece_moving(board))
    }  else if killers[0] == mv {
        PRIMARY_KILLER
    } else if killers[1] == mv {
        SECONDARY_KILLER
    } else {
        history.get_value(&mv, board.turn_color())
    };

    // The move rating is negated so that higher rated moves go first
    RatedMove { mv, score: -score }
}

// Tables for Most Valuable Victim - Least Valuable Aggressor (MVV-LVA)
// Attempts to provide a heuristic for capturing moves by
// capturing with the least valuable piece
const fn value_of_attacker(piece: PieceType) -> MoveScore {
    match piece {
        PieceType::Pawn => 10,
        PieceType::Knight => 30,
        PieceType::Bishop => 30,
        PieceType::Rook => 50,
        PieceType::Queen => 90,
        PieceType::King => 99,
    }
}

const fn value_of_capture(piece: PieceType) -> MoveScore {
    match piece {
        PieceType::Pawn => 100,
        PieceType::Knight => 300,
        PieceType::Bishop => 300,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::King => 9999, // Doesn't happen since the king is never captured
    }
}
