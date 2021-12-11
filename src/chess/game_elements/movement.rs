use std::fmt::{Display, Formatter};
use rocket::serde::{Serialize, Deserialize, Serializer, Deserializer};

use super::{PieceType, Square};

// Avoid clashes between the core Result and the formatter Result
type StdResult<T, E> = core::result::Result<T, E>;
type FmtResult = std::fmt::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Normal { piece: PieceType, from: u8, to: u8, ep: bool},
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

    pub fn piece_type(&self) -> PieceType {
        match self {
            Self::Normal { piece, .. } => *piece,
            Self::PawnPromotion { .. } => PieceType::Pawn,
            _ => unimplemented!()
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
/*

impl<'a> Deserialize<'a> for Move {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> StdResult<Self, D::Error> {
        let s: &str = Deserialize::deserialize(deserializer)?;

        // Is it a castling move?
        if s == "O-O" || s == "0-0" {
            return Ok(Self::ShortCastle);
        } else if s == "O-O-O" || s == "0-0-0" {
            return Ok(Self::LongCastle);
        }

        let len = s.len();

        // Is it a normal move or a promotion move?
        if len == 4  || len == 6 {
            let from = match Position::from_notation(&s[0..2]) {
                Ok(pos) => pos,
                Err(_) => return Err(UnknownMove{s}).map_err(serde::de::Error::custom),
            };

            let to = match Position::from_notation(&s[2..4]) {
                Ok(pos) => pos,
                Err(_) => return Err(UnknownMove{s}).map_err(serde::de::Error::custom),
            };

            // If it's a normal move, it's all good
            if len == 4 {
                return Ok(Move::NormalMove{ from, to });
            }

            // Otherwise, it's a promotion move and the piece must be one of
            // the allowed promotion pieces
            let promote_to = match &s[5..6] {
                "Q" | "q" => PieceType::Queen, 
                "R" | "r" => PieceType::Rook, 
                "B" | "b" => PieceType::Bishop, 
                "N" | "n" => PieceType::Knight, 
                 _  => return Err(UnknownMove{s}).map_err(serde::de::Error::custom)
            };

            return Ok(Move::PawnPromotion { from, to, promote_to });
        }

        // Otherwise, we don't know what the hell this is
        Err(UnknownMove{s}).map_err(serde::de::Error::custom)
    }
}

// Deserializing error
#[derive(Debug)]
struct UnknownMove<'a> {
    s: &'a str
}

impl<'a> std::error::Error for UnknownMove<'a> {}

impl<'a> Display for UnknownMove<'a> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "The move '{}' was not understood or is not valid", self.s)
    }        
}
*/