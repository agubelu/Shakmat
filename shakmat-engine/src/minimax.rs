use std::cmp::{Ordering, max, min};
use std::fmt::{Display, Formatter};
use shakmat_core::{Board, Color::{self, *}, Move};

use crate::evaluation::evaluate_position;
use crate::trasposition::{TTable, TTEntry};

const TRASPOSITION_TABLE_SIZE: usize = 1 << 22;

// Aux types for minimax-related functions
type BestUpdaterFunc = fn(MiniMaxResult, MiniMaxResult) -> MiniMaxResult;
type AlphaBetaUpdaterFunc = fn(&mut i16, &mut i16, i16);
type PruneCheckerFunc = fn(i16, i16, i16) -> bool;

#[derive(Copy, Clone)]
pub struct MiniMaxResult {
    r#move: Option<Move>,
    evaluation: Evaluation
}

#[derive(Copy, Clone)]
pub enum Evaluation {
    Normal { score: i16 },
    ForcedMate { score: i16, losing_color: Color}
} 

// Wrapper function over the minimax algorithm
pub fn find_best(board: &Board, depth: u8) -> MiniMaxResult {
    minimax(board, depth, 0, i16::MIN, i16::MAX, &TTable::new(TRASPOSITION_TABLE_SIZE))
}

fn minimax(board: &Board, depth_remaining: u8, current_depth: u8, mut alpha: i16, mut beta: i16, table: &TTable) -> MiniMaxResult {
    // Check if the current position is in the trasposition table
    let zobrist = board.zobrist_key();
    let tt_entry = unsafe {
        table.get_entry(zobrist).assume_init()
    };

    // If the stored key matches, and the move was searched for a higher depth remaining,
    // use that information. Note that checking whether they key matches also checks
    // whether the entry cointains valid information and not just garbage
    if tt_entry.zobrist() == zobrist {
        let tt_data = unsafe {
            tt_entry.data().assume_init()
        };

        if tt_data.depth >= depth_remaining {
            return tt_data.eval;
        }
    }

    // We use pseudolegal moves, and then check their validity when we generate the
    // new board using that move. This avoids making that check twice, since board.legal_moves()
    // also creates the new board, looks for checks, and then discards it 
    if depth_remaining == 0 {
        let res = MiniMaxResult::new(None, Evaluation::normal(evaluate_position(board)));
        unsafe {
            table.write_entry(zobrist, TTEntry::new(zobrist, depth_remaining, res));
        }
        return res;
    }

    let color_moving = board.turn_color();
    let (initial_val, update_best, update_ab, check_prune) = get_aux_funcs(color_moving);
    let mut best = MiniMaxResult::new(None, initial_val);
        
    for mv in board.pseudolegal_moves() {
        let next_board = board.make_move(&mv, false).unwrap();

        // This is a pseudo-legal move, we must make sure that the side moving is not in check.
        // Castling moves are always legal, for anything else, we must check that the moving side isn't in check
        if matches!(mv, Move::Normal{..} | Move::PawnPromotion{..}) && next_board.is_check(color_moving) {
            continue;
        }

        let next_res = MiniMaxResult::new(
            Some(mv), 
            minimax(&next_board, depth_remaining - 1, current_depth + 1, alpha, beta, table).evaluation
        );
        best = update_best(best, next_res);
        let best_score = best.score();
        update_ab(&mut alpha, &mut beta, best_score);

        if check_prune(alpha, beta, best_score) {
            break;
        }
    }

    // There is a chance that we reached here with a None move, which means that there were
    // no legal moves. In that case, check whether it's a checkmate or a draw, and
    // update the score accordingly
    if best.r#move.is_none() {
        let eval = if board.is_check(color_moving) {
            Evaluation::mate(current_depth as i16, color_moving)
        } else {
            Evaluation::normal(0)
        };

        best.evaluation = eval;
    }

    // Store it in the trasposition table
    unsafe {
        table.write_entry(zobrist, TTEntry::new(zobrist, depth_remaining, best));
    }

    best
}

///////////////////////////////////////////////////////////////////////////////

impl MiniMaxResult {
    fn new(r#move: Option<Move>, evaluation: Evaluation) -> Self {
        Self { r#move, evaluation }
    }

    fn max(self, other: Self) -> Self {
        match self.score().cmp(&other.score()) {
            Ordering::Less => other,
            _ => self,
        }
    }

    fn min(self, other: Self) -> Self {
        match self.score().cmp(&other.score()) {
            Ordering::Greater => other,
            _ => self,
        }
    }

    pub fn get_move(&self) -> Option<Move> {
        self.r#move
    }

    pub fn evaluation(&self) -> &Evaluation {
        &self.evaluation
    }

    pub fn score(&self) -> i16 {
        self.evaluation.score()
    }
}

impl Evaluation {
    pub fn normal(score: i16) -> Self {
        Self::Normal{score}
    }

    pub fn mate(in_turns: i16, losing_color: Color) -> Self {
        let score = match losing_color {
            White => i16::MIN + in_turns,
            Black => i16::MAX - in_turns,
        };
        Self::ForcedMate{score, losing_color}
    }

    pub fn score(&self) -> i16 {
        match &self {
            Evaluation::Normal { score } => *score,
            Evaluation::ForcedMate { score, .. } => *score
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Aux functions to avoid duplicating the minimax code for the max/min players
fn get_aux_funcs(color: Color) -> (Evaluation, BestUpdaterFunc, AlphaBetaUpdaterFunc, PruneCheckerFunc) {
    match color {
        White => (Evaluation::normal(i16::MIN), update_best_max, update_alpha, check_prune_max),
        Black => (Evaluation::normal(i16::MAX), update_best_min, update_beta, check_prune_min),
    }
}

fn update_best_max(current_best: MiniMaxResult, candidate: MiniMaxResult) -> MiniMaxResult {
    current_best.max(candidate)
}

fn update_best_min(current_best: MiniMaxResult, candidate: MiniMaxResult) -> MiniMaxResult {
    current_best.min(candidate)
}

fn update_alpha(alpha: &mut i16, _beta: &mut i16, score: i16) {
    *alpha = max(*alpha, score);
}

fn update_beta(_alpha: &mut i16, beta: &mut i16, score: i16) {
    *beta = min(*beta, score);
}

fn check_prune_max(_alpha: i16, beta: i16, score: i16) -> bool {
    score >= beta
}

fn check_prune_min(alpha: i16, _beta: i16, score: i16) -> bool {
    score <= alpha
}

///////////////////////////////////////////////////////////////////////////////
/// Trait implementations
impl Ord for MiniMaxResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score().cmp(&other.score())
    }
}

impl PartialOrd for MiniMaxResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score().partial_cmp(&other.score())
    }
}

impl Eq for MiniMaxResult {}

impl PartialEq for MiniMaxResult {
    fn eq(&self, other: &Self) -> bool {
        self.score() == other.score()
    }
}

impl Display for Evaluation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Evaluation::Normal{score} => write!(f, "{:+.2}", *score as f32 / 100.0),
            Evaluation::ForcedMate{losing_color, score} => match losing_color {
                White => write!(f, "-M{}", score - i16::MIN),
                Black => write!(f, "M{}", i16::MAX - score),
            }
        }
    }
}