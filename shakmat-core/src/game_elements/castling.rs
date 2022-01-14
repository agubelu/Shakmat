use super::Color;

#[derive(Clone, Copy)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl Default for CastlingRights {
    fn default() -> Self {
        CastlingRights {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }
}

impl CastlingRights {
    pub fn new(white_kingside: bool, white_queenside: bool, black_kingside: bool, black_queenside: bool) -> Self {
        CastlingRights {
            white_kingside,
            white_queenside,
            black_kingside,
            black_queenside,
        }
    }

    pub fn none() -> Self {
        Self::new(false, false, false, false)
    }

    pub fn update_kingside(&mut self, color: Color, can_castle: bool) {
        match color {
            Color::White => self.white_kingside = can_castle,
            Color::Black => self.black_kingside = can_castle,
        }
    }

    pub fn update_queenside(&mut self, color: Color, can_castle: bool) {
        match color {
            Color::White => self.white_queenside = can_castle,
            Color::Black => self.black_queenside = can_castle,
        }
    }

    pub fn disable_all(&mut self, color: Color) {
        match color {
            Color::White => {
                self.white_kingside = false;
                self.white_queenside = false;
            },
            Color::Black => {
                self.black_kingside = false;
                self.black_queenside = false;
            }
        }
    }

    pub fn can_castle_kingside(&self, color: Color) -> bool {
        match color {
            Color::White => self.white_kingside,
            Color::Black => self.black_kingside,
        }
    }


    pub fn can_castle_queenside(&self, color: Color) -> bool {
        match color {
            Color::White => self.white_queenside,
            Color::Black => self.black_queenside,
        }
    }
}