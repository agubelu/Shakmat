mod data_structures;

use data_structures::{TurnInfo, SearchResult};
use shakmat_engine::{ShakmatEngine, EngineConfig, SearchOptions};
use shakmat_core::{Board, Move};
use wasm_bindgen::prelude::*;

//extern crate console_error_panic_hook;

/** 
   Obtains the current turn information for a provided FEN and move history.

   **It is assumed that the FEN is valid.**
*/
#[wasm_bindgen]
pub fn get_turn_data(fen: &str, history: Box<[u64]>) -> TurnInfo {
    let board = Board::from_fen(fen).unwrap();
    TurnInfo::from_board(&board, &history)
}

/** 
   Instructs the engine to look for the best move in a given position by a FEN.

   **It is assumed that the FEN is valid.**
*/
#[wasm_bindgen]
pub fn get_computer_move(
    fen: &str, 
    history: Box<[u64]>,
    move_ms: u32,
    use_opening_book: bool,
    only_best_book_moves: bool
) -> SearchResult {
    let board = Board::from_fen(fen).unwrap();
    let engine_config = EngineConfig { use_opening_book, only_best_book_moves };
    let search_options = SearchOptions { 
        max_depth: None, 
        moves_until_control: None, 
        total_time_remaining: None, 
        time_for_move: Some(move_ms as u64) 
    };

    let engine = ShakmatEngine::new(engine_config);
    let search_data = engine.find_best_move(&board, &history, search_options);

    SearchResult { 
        best_move: search_data.best_move.map(|mv| mv.to_string()), 
        eval: search_data.score.to_string()
    }
}

/** 
   Turns a position encoded by its FEN to its corresponding Zobrist hash.
   This is used to help the client keep track of the previous positions,
   since they must be provided in every request to check for draws by repetition.

   **It is assumed that the FEN is valid.**
*/
#[wasm_bindgen]
pub fn fen2hash(fen: &str) -> u64 {
    Board::from_fen(fen).unwrap().zobrist_key()
}

/** 
    Applies a given move to a given position, returning the FEN string
    for the new position.

    **It is assumed that the move is valid and legal for the position.**
*/
#[wasm_bindgen]
pub fn make_move(fen: &str, movement: &str) -> String {
    let parsed_move = Move::from_notation(movement).unwrap();
    let board = Board::from_fen(fen).unwrap();
    let new_board = board.make_move(&parsed_move);
    new_board.fen()
}
