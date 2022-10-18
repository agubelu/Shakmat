#![feature(adt_const_params)]

mod engine;
mod evaluation;
mod polyglot;
mod search;
mod time;
mod trasposition;

// Exports
pub use search::{is_draw_by_repetition, SearchResult, SearchOptions};
pub use engine::{ShakmatEngine, EngineConfig};
pub use evaluation::init_evaluation;