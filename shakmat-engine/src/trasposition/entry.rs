use std::mem::MaybeUninit;
use crate::search::MiniMaxResult;

#[derive(Copy, Clone)]
pub struct TTEntry {
    zobrist: u64,
    data: MaybeUninit<TTData>,
}

#[derive(Copy, Clone)]
pub struct TTData {
    pub depth: u8,
    pub eval: MiniMaxResult,
}

impl TTEntry {
    pub fn new(zobrist: u64, depth: u8, eval: MiniMaxResult) -> Self {
        let data = MaybeUninit::new(TTData { depth, eval});
        Self { zobrist, data }
    }

    pub fn zobrist(&self) -> u64 {
        self.zobrist
    }

    pub fn data(&self) -> MaybeUninit<TTData> {
        self.data
    }
}