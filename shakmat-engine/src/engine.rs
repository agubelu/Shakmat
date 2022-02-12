use shakmat_core::Board;
use crate::search;
use crate::search::SearchResult;

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

    pub fn find_best_move(&self, board: &Board, past_positions: &[u64]) -> SearchResult {
        let result = search::find_best(board, self.max_depth, past_positions);
        println!("Evaluation: {}", result.score);
        result
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