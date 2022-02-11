use std::fmt::{Display, Formatter};
use serde::{Serialize, Serializer};

use super::{PieceType, Square, PieceType::*};
use crate::board::{Board, BitBoard};

// Avoid clashes between the core Result and the formatter Result
type StdResult<T, E> = core::result::Result<T, E>;
type FmtResult = std::fmt::Result;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Normal { from: u8, to: u8 },
    PawnPromotion { from: u8, to: u8, promote_to: PieceType },
    ShortCastle,
    LongCastle
}

impl Move {
    pub fn to(&self) -> u8 {
        match self {
            Self::Normal { to, .. } => *to,
            Self::PawnPromotion { to, .. } => *to,
            _ => unimplemented!()
        }
    }

    pub fn from(&self) -> u8 {
        match self {
            Self::Normal { from, .. } => *from,
            Self::PawnPromotion { from, .. } => *from,
            _ => unimplemented!()
        }
    }

    pub fn is_capture(&self, board: &Board) -> bool {
        // A move is a capture if the destination square is occupied,
        // of if it's an en passant pawn capture
        match self {
            Self::Normal {to, ..} => !(BitBoard::from_square(*to) & (board.get_all_bitboard() | board.ep_square())).is_empty(),
            Self::PawnPromotion {to, ..} => !(BitBoard::from_square(*to) & board.get_all_bitboard()).is_empty(),
            _ => false
        }
    }

    pub fn piece_moving(&self, board: &Board) -> PieceType {
        match self {
            Self::Normal {from, ..} => board.piece_on(*from).unwrap(),
            Self::PawnPromotion {..} => Pawn,
            _ => King // Castling
        }
    }

    pub fn piece_captured(&self, board: &Board) -> Option<PieceType> {
        match self {
            Self::Normal {to, ..} => *board.piece_on(*to),
            Self::PawnPromotion {to, ..} => *board.piece_on(*to),
            _ => None // Castling
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::Normal { from, to, .. } => write!(f, "{}{}", Square::new(*from), Square::new(*to)),
            Self::ShortCastle => write!(f, "O-O"),
            Self::LongCastle => write!(f, "O-O-O"),
            Self::PawnPromotion { from, to, promote_to } => write!(f, "{}{}={}", 
                Square::new(*from), 
                Square::new(*to), 
                match promote_to {
                    PieceType::Queen => "Q",
                    PieceType::Rook => "R",
                    PieceType::Bishop => "B",
                    PieceType::Knight => "N",
                    _ => unreachable!()
                }),
        }
    }
}

// Custom serialization and deserialization, following the previous formatting
impl Serialize for Move {
    fn serialize<S: Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
