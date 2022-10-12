use std::fmt::{Display, Formatter};
use serde::{Serialize, Serializer};

use super::{PieceType, Square, PieceType::*};
use crate::board::{Board, BitBoard};

// Avoid clashes between the core Result and the formatter Result
type FmtResult = std::fmt::Result;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Normal { from: u8, to: u8 },
    PawnPromotion { from: u8, to: u8, promote_to: PieceType },
    ShortCastle,
    LongCastle
}

impl Move {
    pub fn empty() -> Self {
        // An invalid move, just to use as placeholder and avoid Options
        Self::Normal { from: 0, to: 0 }
    }

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
        // TO-DO: Check if this is faster than using board.piece_on()
        match self {
            Self::Normal {to, ..} => (BitBoard::from_square(*to) & (board.get_all_bitboard() | board.ep_square())).is_not_empty(),
            Self::PawnPromotion {to, ..} => (BitBoard::from_square(*to) & board.get_all_bitboard()).is_not_empty(),
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

    pub fn from_notation(pos: &str) -> Result<Self, String> {
        match pos {
            "O-O" | "0-0" => Ok(Self::ShortCastle),
            "O-O-O" | "0-0-0" => Ok(Self::LongCastle),
            _ if pos.len() >= 4 => {
                let from = Square::from_notation(&pos[0..2])?.square();
                let to = Square::from_notation(&pos[2..4])?.square();

                if pos.len() == 4 {
                    Ok(Self::Normal{from, to})
                } else {
                    let promote_to = match pos[4..].to_lowercase().as_str() {
                        "q" | "=q" => Queen,
                        "r" | "=r" => Rook,
                        "b" | "=b" => Bishop,
                        "n" | "=n" => Knight,
                        _  => return Err("Invalid move".to_owned()),
                    };

                    Ok(Self::PawnPromotion{from, to, promote_to})
                }
            },
            _ => Err("Invalid move".to_owned()),
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::Normal { from, to, .. } => write!(f, "{}{}", Square::new(*from), Square::new(*to)),
            Self::ShortCastle => write!(f, "O-O"),
            Self::LongCastle => write!(f, "O-O-O"),
            Self::PawnPromotion { from, to, promote_to } => write!(f, "{}{}{}", 
                Square::new(*from), 
                Square::new(*to), 
                match promote_to {
                    PieceType::Queen => "q",
                    PieceType::Rook => "r",
                    PieceType::Bishop => "b",
                    PieceType::Knight => "n",
                    _ => unreachable!()
                }),
        }
    }
}

// Custom serialization and deserialization, following the previous formatting
impl Serialize for Move {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
