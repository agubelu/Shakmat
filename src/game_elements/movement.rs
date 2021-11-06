use super::{Position, PieceType};

#[derive(Debug, Clone, Copy)]
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