mod engine;
mod evaluation;
mod polyglot;
mod search;
mod time;
mod trasposition;

// Exports
pub use search::{is_draw_by_repetition, SearchResult, SearchOptions};
pub use engine::ShakmatEngine;