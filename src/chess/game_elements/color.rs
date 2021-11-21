use std::ops::Not;
use rocket::serde::Serialize;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Color {
    White,
    Black,
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

