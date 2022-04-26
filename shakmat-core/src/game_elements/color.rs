use std::ops::Not;
use serde::Serialize;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
pub enum Color {
    White,
    Black,
}

impl Color {
    // Used for zobrist keys and array indexing
    pub const fn to_index(&self) -> usize {
        match self {
            Self::Black => 0,
            Self::White => 1,
        }
    }

    // Used as a multiplier to swap the sign in score calculations
    pub const fn sign(&self) -> i16 {
        match self {
            Self::White =>  1,
            Self::Black => -1,
        }
    }
}

impl Not for Color {
    type Output = Color;

    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

