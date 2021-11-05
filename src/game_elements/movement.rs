use super::Position;

#[derive(Debug, Clone, Copy)]
pub enum Move {
    NormalMove { from: Position, to: Position },
    ShortCastle,
    LongCastle,
}