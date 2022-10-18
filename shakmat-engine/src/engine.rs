use shakmat_core::Board;

use crate::evaluation::Evaluation;
use crate::polyglot::OpeningBook;
use crate::search::{SearchResult, SearchOptions, Search};

pub struct ShakmatEngine {
    book: OpeningBook,
    config: EngineConfig,
}

pub struct EngineConfig {
    pub use_opening_book: bool,
    pub only_best_book_moves: bool,
}

impl ShakmatEngine {
    pub fn new(config: EngineConfig) -> Self {
        Self { config, book: OpeningBook::load() }
    }

    pub fn find_best_move(&self, board: &Board, past_positions: &[u64], options: SearchOptions) -> SearchResult {
        if self.config.use_opening_book {
            // Query our opening book to get a move for this position
            if let Some(mv) = self.book.get_move(board, self.config.only_best_book_moves) {
                // We know this opening line, play the move from the book
                println!("Book move");
                return SearchResult { best_move: Some(mv), score: Evaluation::new(0) }
            }
        }

        // Otherwise do a normal search for the best move
        let result = Search::from_config(options, past_positions).find_best(board);
        println!("Evaluation: {}", result.score);
        result
    }

    pub fn update_config(&mut self, config: EngineConfig) {
        self.config = config;
    }
}

impl Default for ShakmatEngine {
    fn default() -> Self {
        Self::new(EngineConfig::default())
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self { only_best_book_moves: true, use_opening_book: true }
    }
}