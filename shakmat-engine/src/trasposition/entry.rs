use std::mem::MaybeUninit;
use shakmat_core::Move;
use crate::evaluation::Evaluation;

#[derive(Copy, Clone)]
pub struct TTEntry {
    zobrist: u64,
    data: MaybeUninit<TTData>,
}

#[derive(Copy, Clone)]
pub struct TTData {
    pub depth: u8,
    pub eval: Evaluation,
    pub node_type: NodeType,
    pub best_move: Option<Move>
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum NodeType {
    Exact, AlphaCutoff, BetaCutoff
}

impl TTEntry {
    pub fn new(zobrist: u64, depth: u8, eval: Evaluation, node_type: NodeType, best_move: Option<Move>) -> Self {
        let data = MaybeUninit::new(TTData { depth, eval, node_type, best_move });
        Self { zobrist, data }
    }

    pub fn zobrist(&self) -> u64 {
        self.zobrist
    }

    pub fn data(&self) -> MaybeUninit<TTData> {
        self.data
    }
}

impl TTData {
    pub fn eval_score(&self) -> Evaluation {
        self.eval
    }

    pub fn node_type(&self) -> NodeType {
        self.node_type
    }

    pub fn best_move(&self) -> &Option<Move> {
        &self.best_move
    }
}