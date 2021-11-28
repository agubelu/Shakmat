use std::fmt::Display;
use std::ops::{BitAnd, BitOr, BitOrAssign, Not};
use std::cmp::PartialEq;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BitBoard {
    bb: u64
}

pub struct PieceIndexIter {
    bb: u64,
    cur_index: u8
}

impl BitBoard {
    pub const fn new(bb: u64) -> Self {
        BitBoard { bb }
    }

    pub fn get_u64(&self) -> u64 {
        self.bb
    }

    pub fn wrapping_mul(&self, other: u64) -> u64 {
        self.bb.wrapping_mul(other)
    }

    pub fn set_one(&mut self, pos: u64) {
        self.bb |= pos;
    }

    pub fn set_zero(&mut self, pos: u64) {
        self.bb &= !pos;
    }

    pub fn reset(&mut self) {
        self.bb = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.bb == 0
    }

    pub fn piece_indices(&self) -> PieceIndexIter {
        PieceIndexIter { bb: self.bb, cur_index: 0 }
    }
}

///////////////////////////////////////////////////////////////////////////////
impl Iterator for PieceIndexIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bb == 0 {
            None
        } else {
            let i = self.bb.trailing_zeros() as u8 + 1;
            self.bb >>= i;
            self.cur_index += i;
            Some(self.cur_index - 1)
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Aux trait implements for BB

impl Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.bb.to_be_bytes().iter()
            .map(|b| format!("{:#010b}", b))
            .map(|bin| bin[2..].replace("0", "."))
            .fold(String::new(), |a, b| format!("{}{}\n", a, b))
            .fmt(f)
    }
}

impl BitAnd<Self> for BitBoard {
    type Output = Self;

    fn bitand(self, other: Self) -> Self::Output {
        Self::new(self.bb & other.bb)
    }
}

impl BitAnd<u64> for BitBoard {
    type Output = Self;

    fn bitand(self, other: u64) -> Self::Output {
        Self::new(self.bb & other)
    }
}

impl BitOr<Self> for BitBoard {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        Self::new(self.bb | other.bb)
    }
}

impl BitOrAssign<Self> for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.bb |= rhs.bb;
    }
}

impl BitOr<u64> for BitBoard {
    type Output = Self;

    fn bitor(self, other: u64) -> Self::Output {
        Self::new(self.bb | other)
    }
}

impl Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::new(!self.bb)
    }
}

impl Default for BitBoard {
    fn default() -> Self {
        Self::new(0)
    }
}