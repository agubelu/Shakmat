use std::sync::Mutex;
use std::mem::drop;

use rocket::serde::json::Json;
use rocket::{Route, State};
use shakmat_engine::find_best_move;

use crate::messages::{ApiResponse, FenData, MoveData};
use crate::state::ServerState;

type StateMutex = State<Mutex<ServerState>>;

pub fn get_routes() -> Vec<Route> {
    routes![create_game, get_turn_info, make_move, get_computer_move]
}

#[post("/games", data = "<fen>")]
pub fn create_game(state: &StateMutex, fen: Option<Json<FenData>>) -> ApiResponse {
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
pub fn get_turn_info(state: &StateMutex, game_id: &str) -> ApiResponse {
    let state_lock = state.inner().lock().unwrap();
    match state_lock.get_turn_info(game_id) {
        Some(turn_info) => ApiResponse::turn_info(turn_info),
        None => ApiResponse::not_found("Game not found".to_owned()),
    }
}

#[post("/games/<game_id>/move", data = "<move>")]
pub fn make_move(state: &StateMutex, game_id: &str, r#move: Json<MoveData>) -> ApiResponse {
    let mut state_lock = state.inner().lock().unwrap();

    let mv = match state_lock.get_game_moves(game_id) {
        None => return ApiResponse::not_found("Game not found".to_owned()),
        Some(map) => match map.get(&r#move.r#move.to_lowercase()) {
            Some(movement) => *movement,
            None => return ApiResponse::bad_request("Illegal move".to_owned()),
        }
    };

    match state_lock.make_move(game_id, mv) {
        Ok(()) => ApiResponse::turn_info(state_lock.get_turn_info(game_id).unwrap()),
        Err(msg) => ApiResponse::bad_request(msg),
    }
}

#[get("/games/<game_id>/move_suggestion")]
pub fn get_computer_move(state: &StateMutex, game_id: &str) -> ApiResponse {
    let state_lock = state.inner().lock().unwrap();
    let board = match state_lock.get_board(game_id) {
        Some(board) => *board,
        None => return ApiResponse::not_found("Game not found".to_owned()),
    };

    // We drop the lock here so the rather slow process of finding the best
    // move doesn't block all othe requests
    drop(state_lock);

    match find_best_move(&board) {
        Some(mv) => ApiResponse::move_suggestion(&mv),
        None => ApiResponse::bad_request("No moves available".to_owned()),
    }
}