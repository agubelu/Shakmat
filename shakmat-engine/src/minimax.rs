use std::cmp::{Ordering, max, min};
use std::fmt::{Display, Formatter};
use shakmat_core::{Board, Color::{self, *}, Move};
use crate::evaluation::evaluate_position;

// Aux types for minimax-related functions
type BestUpdaterFunc = fn(MiniMaxResult, MiniMaxResult) -> MiniMaxResult;
type AlphaBetaUpdaterFunc = fn(&mut i32, &mut i32, i32);
type PruneCheckerFunc = fn(i32, i32, i32) -> bool;

pub struct MiniMaxResult {
    r#move: Option<Move>,
    evaluation: Evaluation
}

pub enum Evaluation {
    Normal { score: i32 },
    ForcedMate { score: i32, losing_color: Color}
} 

// Wrapper function over the minimax algorithm
pub fn find_best(board: &Board, depth: u16) -> MiniMaxResult {
    minimax(board, depth, 0, i32::MIN, i32::MAX)
}

fn minimax(board: &Board, depth_remaining: u16, current_depth: u16, mut alpha: i32, mut beta: i32) -> MiniMaxResult {
    // We use pseudolegal moves, and then check their validity when we generate the
    // new board using that move. This avoids making that check twice, since board.legal_moves()
    // also creates the new board, looks for checks, and then discards it 
    if depth_remaining == 0 {
        return MiniMaxResult::new(None, Evaluation::normal(evaluate_position(board)));
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
            minimax(&next_board, depth_remaining - 1, current_depth + 1, alpha, beta).evaluation
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
            Evaluation::mate(current_depth as i32, color_moving)
        } else {
            Evaluation::normal(0)
        };

        best.evaluation = eval;
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

    pub fn score(&self) -> i32 {
        self.evaluation.score()
    }
}

impl Evaluation {
    pub fn normal(score: i32) -> Self {
        Self::Normal{score}
    }

    pub fn mate(in_turns: i32, losing_color: Color) -> Self {
        let score = match losing_color {
            White => i32::MIN + in_turns,
            Black => i32::MAX - in_turns,
        };
        Self::ForcedMate{score, losing_color}
    }

    pub fn score(&self) -> i32 {
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
        White => (Evaluation::normal(i32::MIN), update_best_max, update_alpha, check_prune_max),
        Black => (Evaluation::normal(i32::MAX), update_best_min, update_beta, check_prune_min),
    }
}

fn update_best_max(current_best: MiniMaxResult, candidate: MiniMaxResult) -> MiniMaxResult {
    current_best.max(candidate)
}

fn update_best_min(current_best: MiniMaxResult, candidate: MiniMaxResult) -> MiniMaxResult {
    current_best.min(candidate)
}

fn update_alpha(alpha: &mut i32, _beta: &mut i32, score: i32) {
    *alpha = max(*alpha, score);
}

fn update_beta(_alpha: &mut i32, beta: &mut i32, score: i32) {
    *beta = min(*beta, score);
}

fn check_prune_max(_alpha: i32, beta: i32, score: i32) -> bool {
    score >= beta
}

fn check_prune_min(alpha: i32, _beta: i32, score: i32) -> bool {
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
                White => write!(f, "-M{}", score - i32::MIN),
                Black => write!(f, "M{}", i32::MAX - score),
            }
        }
    }
}