use super::{Position, PieceType};

#[derive(Debug, Clone, Copy)]
pub enum Move {
    NormalMove { from: Position, to: Position },
    ShortCastle,
    LongCastle,
    PawnPromotion { from: Position, to: Position, promote_to: PieceType}
}