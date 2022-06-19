use std::fmt::Display;
use std::ops::{BitAnd, BitOr, BitOrAssign, BitAndAssign, BitXorAssign, Not, Shl, Shr};
use std::cmp::PartialEq;

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct BitBoard {
    bb: u64
}

pub struct PieceIndexIter {
    value: u64,
}

impl BitBoard {
    pub const fn new(bb: u64) -> Self {
        BitBoard { bb }
    }

    pub const fn ones() -> Self {
        Self::new(u64::MAX)
    }

    pub fn from_square(square: u8) -> Self {
        BitBoard { bb: 1 << square }
    }

    pub fn get_u64(&self) -> u64 {
        self.bb
    }

    pub fn wrapping_mul(&self, other: u64) -> u64 {
        self.bb.wrapping_mul(other)
    }

    pub fn clear(&mut self) {
        self.bb = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.bb == 0
    }

    pub fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }

    pub fn piece_indices(&self) -> PieceIndexIter {
        PieceIndexIter { value: self.bb }
    }

    pub fn first_piece_index(&self) -> u8 {
        // Callers assume that there is at least one piece in this BB
        self.bb.trailing_zeros() as u8
    }

    pub fn count(&self) -> u32 {
        self.bb.count_ones()
    }
}

///////////////////////////////////////////////////////////////////////////////
impl Iterator for PieceIndexIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.value {
            0 => None,
            x => {
                self.value = x & (x - 1);
                Some(x.trailing_zeros() as u8)
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Aux trait implements for BB

impl Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.bb.to_be_bytes().iter()
            .map(|b| format!("{:#010b}", b))
            .map(|bin| bin[2..].replace('0', "."))
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

impl BitAndAssign<Self> for BitBoard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.bb &= rhs.bb;
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

impl BitXorAssign<Self> for BitBoard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.bb ^= rhs.bb;
    }
}

impl BitOr<u64> for BitBoard {
    type Output = Self;

    fn bitor(self, other: u64) -> Self::Output {
        Self::new(self.bb | other)
    }
}

impl Shl<u8> for BitBoard {
    type Output = Self;

    fn shl(self, rhs: u8) -> Self::Output {
        BitBoard::new(self.bb << rhs)
    }
}

impl Shr<u8> for BitBoard {
    type Output = Self;

    fn shr(self, rhs: u8) -> Self::Output {
        BitBoard::new(self.bb >> rhs)
    }
}

impl Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::new(!self.bb)
    }
}
