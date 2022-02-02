use std::fmt::{Formatter, Display};
use std::ops::{Neg, Add};
use shakmat_core::{Board, Color::*, Color};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
// Represents the evaluation of a position. The goal of using a struct instead of an i16
// directly is to implement Display, to be able to show the score in a much nicer way
// (for example, plies to checkmate instead of the raw score)
pub struct Evaluation { score: i16 } 

// Evaluate how favorable a position is for the current side to move
// A positive score favors the current side, while a negative one
// favors the rival.
pub fn evaluate_position(board: &Board) -> Evaluation {
    let score = piece_score(board, White) - piece_score(board, Black)
    + control_score(board, White) - control_score(board, Black);

    Evaluation::new(score * board.turn_color().sign())
}

// Computes the total piece score of a color, where:
// - Pawns: 100 point
// - Knights and Bishops: 300 points
// - Rooks: 500 points
// - Queen: 900 points
fn piece_score(board: &Board, color: Color) -> i16 {
    let pieces = board.get_pieces(color);
    
    let score = 100 * pieces.pawns.count() +
    300 * pieces.knights.count() +
    300 * pieces.bishops.count() +
    500 * pieces.rooks.count() +
    900 * pieces.queens.count();

    score as i16
}

fn control_score(board: &Board, color: Color) -> i16 {
    board.get_attack_bitboard(color).count() as i16 * 5
}

impl Evaluation {
    pub fn new(score: i16) -> Self {
        Self { score }
    }

    // The min value is set to i16::MIN + 1, so that -min_val() == max_val()
    // and viceversa. Otherwise, it overflows when swapping its sign
    // and all sort of bad things happen.
    pub fn min_val() -> Self {
        Self::new(i16::MIN + 1)
    }

    pub fn max_val() -> Self {
        Self::new(i16::MAX)
    }

    pub fn score(&self) -> i16 {
        self.score
    }
}

impl Neg for Evaluation {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.score)
    }
}

impl Add<i16> for Evaluation {
    type Output = Self;

    fn add(self, rhs: i16) -> Self::Output {
        Self::new(self.score + rhs)
    }
}

impl Display for Evaluation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.score() > i16::MAX - 100 {
            write!(f, "M{}", i16::MAX - self.score())
        } else if self.score() < i16::MIN + 100 {
            write!(f, "-M{}", self.score() - i16::MIN - 1)
        } else {
            write!(f, "{:+.2}", self.score() as f32 / 100.0)
        }
    }
}