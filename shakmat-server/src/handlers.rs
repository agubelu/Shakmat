use std::sync::Mutex;
use std::mem::drop;

use shakmat_core::Move;
use shakmat_engine::{ShakmatEngine, SearchOptions, EngineConfig};
use rocket::serde::json::Json;
use rocket::{Route, State};

use crate::messages::{ApiResponse, FenData, MoveData, ConfigOptions};
use crate::state::ServerState;

type StateMutex<T> = State<Mutex<T>>;
type GamesState = StateMutex<ServerState>;
type EngineState = StateMutex<ShakmatEngine>;

pub fn get_routes() -> Vec<Route> {
    routes![create_game, get_turn_info, make_move, get_computer_move, delete_game, config_engine, _all_options]
}

// Catches all OPTION requests in order to get the CORS related Fairing triggered.
// Thanks to this absolute hero: https://stackoverflow.com/a/72702246/5604339
#[options("/<_..>")]
fn _all_options() { /* Intentionally left empty */ }

#[post("/games", data = "<fen>")]
pub fn create_game(state: &GamesState, fen: Option<Json<FenData>>) -> ApiResponse {
    let mut state_lock = state.inner().lock().unwrap();

    let key = if let Some(fen_data) = fen {
        // Create a new game from the supplied FEN
        match state_lock.create_game_from_fen(&fen_data.fen) {
            Ok(key) => key,
            Err(msg) => return ApiResponse::bad_request(msg),
        }
    } else {
        // Create a default game
        state_lock.create_game_default()
    };

    // We can unwrap the option because we know the key exists, since we
    // just created it
    let turn_info = state_lock.get_turn_info(&key).unwrap();
    ApiResponse::game_created(key, turn_info)
}

#[get("/games/<game_id>")]
pub fn get_turn_info(state: &GamesState, game_id: &str) -> ApiResponse {
    let state_lock = state.inner().lock().unwrap();
    match state_lock.get_turn_info(game_id) {
        Some(turn_info) => ApiResponse::turn_info(turn_info),
        None => ApiResponse::not_found("Game not found".to_owned()),
    }
}

#[post("/games/<game_id>/move", data = "<move>")]
pub fn make_move(state: &GamesState, game_id: &str, r#move: Json<MoveData>) -> ApiResponse {
    let mut state_lock = state.inner().lock().unwrap();
    let mv = match Move::from_notation(&r#move.r#move) {
        Ok(m) => m,
        Err(msg) => return ApiResponse::bad_request(msg), 
    };

    match state_lock.make_move(game_id, mv) {
        Ok(()) => ApiResponse::turn_info(state_lock.get_turn_info(game_id).unwrap()),
        Err(msg) => ApiResponse::bad_request(msg),
    }
}

#[get("/games/<game_id>/move_suggestion?<depth>&<move_ms>&<total_ms>")]
pub fn get_computer_move(state: &GamesState, engine: &EngineState, game_id: &str,
depth: Option<u8>, move_ms: Option<u64>, total_ms: Option<u64>) -> ApiResponse {
    let state_lock = state.inner().lock().unwrap();
    let board = match state_lock.get_board(game_id) {
        Some(board) => *board,
        None => return ApiResponse::not_found("Game not found".to_owned()),
    };
    
    // Get the list of past positions (cloning it, since we drop the lock
    // in the next step). We can assume that the game ID exists, otherwise
    // we would have returned a not_found response.
    let past_positions = state_lock.get_history(game_id).unwrap().clone();

    // We drop the lock here so the rather slow process of finding the best
    // move doesn't block all other requests
    drop(state_lock);

    // Create the search options struct with the data from the query string
    let search_options = SearchOptions { 
        total_time_remaining: total_ms,
        moves_until_control: None, //TO-DO
        time_for_move: move_ms,
        max_depth: depth,
    };

    let engine_lock = engine.inner().lock().unwrap();
    let search_result = engine_lock.find_best_move(&board, &past_positions, search_options);

    match search_result.best_move {
        Some(_) => ApiResponse::move_suggestion(&search_result),
        None => ApiResponse::bad_request("No moves available".to_owned()),
    }
}

#[delete("/games/<game_id>")]
pub fn delete_game(state: &GamesState, game_id: &str) -> ApiResponse {
    let mut state_lock = state.inner().lock().unwrap();
    match state_lock.delete_game(game_id) {
        Ok(()) => ApiResponse::no_content(),
        Err(msg) => ApiResponse::not_found(msg),
    }
}

#[post("/config", data = "<config>")]
pub fn config_engine(engine: &EngineState, config: Json<ConfigOptions>) -> ApiResponse {
    let mut state_lock = engine.inner().lock().unwrap();

    let config_engine = EngineConfig {
        use_opening_book: config.use_book,
        only_best_book_moves: config.always_top_line,
    };

    state_lock.update_config(config_engine);
    ApiResponse::no_content()
}