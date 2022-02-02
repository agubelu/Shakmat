use std::fmt::{Display, Formatter};
use serde::{Serialize, Serializer};

use super::{PieceType, Square};

// Avoid clashes between the core Result and the formatter Result
type StdResult<T, E> = core::result::Result<T, E>;
type FmtResult = std::fmt::Result;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Normal { from: u8, to: u8, ep: bool},
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

    pub fn is_ep(&self) -> bool {
        match self {
            Self::Normal {ep, ..} => *ep,
            _ => false
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
