mod evaluation;
mod search;
mod trasposition;
mod engine;

// Exports
pub use search::{is_draw_by_repetition, SearchResult};
pub use engine::ShakmatEngine;