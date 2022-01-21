use std::cmp::{Ordering, max, min};
use shakmat_core::{Board, Color::*, Move};
use crate::evaluation::evaluate_position;

#[derive(Default)]
pub struct MiniMaxResult {
    r#move: Option<Move>,
    score: i32
}

// Wrapper function over the minimax algorithm
pub fn find_best(board: &Board, depth: u16) -> MiniMaxResult {
    minimax(board, depth, i32::MIN, i32::MAX)
}

fn minimax(board: &Board, depth: u16, mut alpha: i32, mut beta: i32) -> MiniMaxResult {
    let moves = board.legal_moves();

    if depth == 0 || moves.is_empty() {
        return MiniMaxResult::new(None, evaluate_position(board, &moves));
    }

    if board.turn_color() == White {
        let mut best = MiniMaxResult::new(None, i32::MIN);
        
        for mv in moves {
            let next_board = board.make_move(&mv, false).unwrap();
            let next_res = MiniMaxResult::new(Some(mv), minimax(&next_board, depth - 1, alpha, beta).score);

            best = best.max(next_res);
            alpha = max(alpha, best.score());

            if best.score() >= beta {
                break;
            }
        }

        best
    } else {
        let mut best = MiniMaxResult::new(None, i32::MAX);
        
        for mv in moves {
            let next_board = board.make_move(&mv, false).unwrap();
            let next_res = MiniMaxResult::new(Some(mv), minimax(&next_board, depth - 1, alpha, beta).score);

            best = best.min(next_res);
            beta = min(beta, best.score());
            
            if best.score() <= alpha {
                break;
            }
        }

        best
    }

    /*
    let minimaxed_moves = moves.into_iter()
        .map(|mv| {
            let new_board = board.make_move(&mv, false).unwrap();
            MiniMaxResult::new(Some(mv), minimax(&new_board, depth - 1, alpha, beta).score)
        });

    let best = match board.turn_color() {
        White => minimaxed_moves.max(),
        Black => minimaxed_moves.min(),
    };

    best.unwrap_or_default()
    
    */

    
}

///////////////////////////////////////////////////////////////////////////////

impl MiniMaxResult {
    fn new(r#move: Option<Move>, score: i32) -> Self {
        Self { r#move, score }
    }

    fn max(self, other: Self) -> Self {
        match self.score.cmp(&other.score) {
            Ordering::Less => other,
            _ => self
        }
    }

    fn min(self, other: Self) -> Self {
        match self.score.cmp(&other.score) {
            Ordering::Greater => other,
            _ => self
        }
    }

    pub fn get_move(&self) -> Option<Move> {
        self.r#move
    }

    pub fn score(&self) -> i32 {
        self.score
    }
}

impl Ord for MiniMaxResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for MiniMaxResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl Eq for MiniMaxResult {}

impl PartialEq for MiniMaxResult {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}