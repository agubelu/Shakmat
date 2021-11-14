use std::fmt::{Display, Formatter, Result};

use super::{Position, PieceType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    NormalMove { from: Position, to: Position },
    ShortCastle,
    LongCastle,
    PawnPromotion { from: Position, to: Position, promote_to: PieceType}
}

impl Move {
    pub fn to(&self) -> &Position {
        match self {
            Move::NormalMove { to, .. } => to,
            Move::PawnPromotion { to, .. } => to,
            _ => unimplemented!()
        }
    }

    pub fn from(&self) -> &Position {
        match self {
            Move::NormalMove { from, .. } => from,
            Move::PawnPromotion { from, .. } => from,
            _ => unimplemented!()
        }
    }       
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Move::NormalMove { from, to } => write!(f, "{}{}", from.as_notation(), to.as_notation()),
            Move::ShortCastle => write!(f, "O-O"),
            Move::LongCastle => write!(f, "O-O-O"),
            Move::PawnPromotion { from, to, promote_to } => write!(f, "{}{}={}", 
                from.as_notation(), 
                to.as_notation(), 
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