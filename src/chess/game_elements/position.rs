use std::fmt::{Display, Formatter};
use std::result::Result;
use crate::chess::BitBoard;

type FmtResult = std::fmt::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square {
    square: u8,
}

impl Square {
    pub fn new(square: u8) -> Self {
        assert!(square < 64);
        Self { square }
    }

    pub fn square(&self) -> u8 {
        self.square
    }

    pub fn file(&self) -> u8 {
        7 - (self.square % 8)
    }

    pub fn rank(&self) -> u8 {
        self.square / 8
    }

    pub fn as_bitboard(&self) -> BitBoard {
        BitBoard::new(1 << self.square)
    }

    pub fn from_file_rank(file: u8, rank: u8) -> Result<Self, String> {
        if file > 7 {
            Err(format!("Invalid file: {}", file))
        } else if rank > 7 {
            Err(format!("Invalid rank: {}", file))
        } else {
            Ok(Self::new(rank * 8 + (7 - file)))
        }
    }

    pub fn from_notation(pos: &str) -> Result<Self, String> {
        let pos_chars: Vec<char> = pos.chars().collect();

        if pos_chars.len() != 2 {
            return Err(format!("Invalid position: {}", pos));
        }

        let file = match pos_chars[0] {
            'a' | 'A' => 0,
            'b' | 'B' => 1,
            'c' | 'C' => 2,
            'd' | 'D' => 3,
            'e' | 'E' => 4,
            'f' | 'F' => 5,
            'g' | 'G' => 6,
            'h' | 'H' => 7,
             x  => return Err(format!("Invalid file: {}", x)),
        };

        let rank = match pos_chars[1] {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            '4' => 3,
            '5' => 4,
            '6' => 5,
            '7' => 6,
            '8' => 7,
             x  => return Err(format!("Invalid rank: {}", x)),
        };

        Ok(Self::from_file_rank(file, rank).unwrap())
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let file = match self.file() {
            0 => "a",
            1 => "b",
            2 => "c",
            3 => "d",
            4 => "e",
            5 => "f",
            6 => "g",
            7 => "h",
            _ => unreachable!()
        };

        let rank = self.rank() + 1;
        format!("{}{}", file, rank).fmt(f)
    }
}
