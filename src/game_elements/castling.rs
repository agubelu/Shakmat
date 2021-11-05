use crate::game_elements::Color;

#[derive(Debug, Clone, Copy)]
pub struct CastlingRights {
    pub WhiteKingSide: bool,
    pub WhiteQueenSide: bool,
    pub BlackKingSide: bool,
    pub BlackQueenSide: bool,
}

impl Default for CastlingRights {
    fn default() -> Self {
        CastlingRights {
            WhiteKingSide: true,
            WhiteQueenSide: true,
            BlackKingSide: true,
            BlackQueenSide: true,
        }
    }
}

impl CastlingRights {
    pub fn new(WhiteKingSide: bool, WhiteQueenSide: bool, BlackKingSide: bool, BlackQueenSide: bool) -> Self {
        CastlingRights {
            WhiteKingSide,
            WhiteQueenSide,
            BlackKingSide,
            BlackQueenSide,
        }
    }

    pub fn none() -> Self {
        Self::new(false, false, false, false)
    }

    pub fn update_kingside(&mut self, color: Color, can_castle: bool) {
        match color {
            Color::White => self.WhiteKingSide = can_castle,
            Color::Black => self.BlackKingSide = can_castle,
        }
    }

    pub fn update_queenside(&mut self, color: Color, can_castle: bool) {
        match color {
            Color::White => self.WhiteQueenSide = can_castle,
            Color::Black => self.BlackQueenSide = can_castle,
        }
    }
}