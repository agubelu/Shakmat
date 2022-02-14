use shakmat_core::Board;

use crate::evaluation::Evaluation;
use crate::polyglot::OpeningBook;
use crate::search;
use crate::search::SearchResult;

pub struct ShakmatEngine {
    book: OpeningBook,
    config: EngineConfig,
}

pub struct EngineConfig {
    max_depth: u8,
    only_best_book_moves: bool,
}

impl ShakmatEngine {
    pub fn new(config: EngineConfig) -> Self {
        Self { config, book: OpeningBook::load() }
    }

    pub fn find_best_move(&self, board: &Board, past_positions: &[u64]) -> SearchResult {
        // Query our opening book to get a move for this position
        if let Some(mv) = self.book.get_move(board, self.config.only_best_book_moves) {
            // We know this opening, play the move from the book
            println!("Book move");
            return SearchResult { best_move: Some(mv), score: Evaluation::new(0) }
        }

        // Otherwise do a normal search for the best move
        let result = search::find_best(board, self.config.max_depth, past_positions);
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
        Self { max_depth: 7, only_best_book_moves: true }
    }
}