use std::cmp::Ordering;
use shakmat_core::{Board, Color::*, Move};
use crate::evaluation::evaluate_position;

#[derive(Default)]
pub struct MiniMaxResult {
    r#move: Option<Move>,
    score: i32
}

pub fn find_best(board: &Board, cur_depth: u16, max_depth: u16) -> MiniMaxResult {
    let moves = board.legal_moves();

    if cur_depth == max_depth || moves.is_empty() {
        return MiniMaxResult::new(None, evaluate_position(board, &moves));
    }

    let minimaxed_moves = moves.into_iter()
        .map(|mv| {
            let new_board = board.make_move(&mv, false).unwrap();
            MiniMaxResult::new(Some(mv), find_best(&new_board, cur_depth + 1, max_depth).score)
        });

    let best = match board.turn_color() {
        White => minimaxed_moves.max(),
        Black => minimaxed_moves.min(),
    };

    best.unwrap_or_default()
}

///////////////////////////////////////////////////////////////////////////////

impl MiniMaxResult {
    fn new(r#move: Option<Move>, score: i32) -> Self {
        Self { r#move, score }
    }

    pub fn get_move(&self) -> Option<Move> {
        self.r#move
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