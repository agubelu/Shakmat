mod engine;
mod evaluation;
mod polyglot;
mod search;
mod trasposition;

// Exports
pub use search::{is_draw_by_repetition, SearchResult};
pub use engine::ShakmatEngine;