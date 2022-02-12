use shakmat_core::{Board, Move};
use crate::search;

pub struct ShakmatEngine {
    // TO-DO: Opening books, config, etc...
    max_depth: u8,
}

pub struct EngineConfig {
    max_depth: u8
}

impl ShakmatEngine {
    pub fn new(config: EngineConfig) -> Self {
        Self { max_depth: config.max_depth }
    }

    pub fn find_best_move(&self, board: &Board, past_positions: &[u64]) -> Option<Move> {
        let best = search::find_best(board, self.max_depth, past_positions);
        println!("Evaluation: {}", best.eval);
        best.best
    }
}

impl Default for ShakmatEngine {
    fn default() -> Self {
        Self::new(EngineConfig::default())
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self { max_depth: 6 }
    }
}