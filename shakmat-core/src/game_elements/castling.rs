use super::Color;

#[derive(Clone, Copy)]
pub struct CastlingRights {
    // We use the last 4 bits of an u8: XXXXABCD
    // A -> White kingside
    // B -> White queenside
    // C -> Black kingside
    // D -> Black queenside
    rights: u8
}

impl Default for CastlingRights {
    fn default() -> Self {
        CastlingRights { rights: 0x0F }
    }
}

impl CastlingRights {
    pub fn new(white_kingside: bool, white_queenside: bool, black_kingside: bool, black_queenside: bool) -> Self {
        CastlingRights { rights:
            (white_kingside as u8) << 3 |
            (white_queenside as u8) << 2 |
            (black_kingside as u8) << 1 |
            (black_queenside as u8)
        }
    }

    pub fn none() -> Self {
        Self::new(false, false, false, false)
    }

    pub const fn index(&self) -> usize {
        self.rights as usize
    }

    pub fn update_kingside(&mut self, color: Color, can_castle: bool) {
        let b = can_castle as u8;
        match color {
            Color::White => self.rights = (self.rights & !(1 << 3)) | (b << 3),
            Color::Black => self.rights = (self.rights & !(1 << 1)) | (b << 1),
        }
    }

    pub fn update_queenside(&mut self, color: Color, can_castle: bool) {
        let b = can_castle as u8;
        match color {
            Color::White => self.rights = (self.rights & !(1 << 2)) | (b << 2),
            Color::Black => self.rights = self.rights & !1 | b,
        }
    }

    pub fn disable_all(&mut self, color: Color) {
        match color {
            Color::White => self.rights &= 0b00000011,
            Color::Black => self.rights &= 0b00001100,
        }
    }

    pub fn has_no_rights(&self) -> bool {
        self.rights == 0
    }

    pub fn can_castle_kingside(&self, color: Color) -> bool {
        match color {
            Color::White => self.rights & 0b00001000 != 0,
            Color::Black => self.rights & 0b00000010 != 0,
        }
    }

    pub fn can_castle_queenside(&self, color: Color) -> bool {
        match color {
            Color::White => self.rights & 0b00000100 != 0,
            Color::Black => self.rights & 0b00000001 != 0,
        }
    }

    pub fn has_lost_rights(&self, color: Color) -> bool {
        match color {
            Color::White => self.rights & 0b00001100 == 0,
            Color::Black => self.rights & 0b00000011 == 0,
        }
    }
}