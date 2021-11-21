use std::sync::Mutex;
use rocket::serde::json::Json;
use rocket::State;

use super::messages::{ApiResponse, FenData};
use super::state::ServerState;

type StateMutex = State<Mutex<ServerState>>;

#[post("/new/default")]
pub fn create_default_game(state: &StateMutex) -> ApiResponse {
    let mut inner_state = state.inner().lock().unwrap();
    let key = inner_state.create_game_default();
    let board = inner_state.get_game(&key).unwrap().lock().unwrap(); 
    println!("Creating a new default game with key: {}", key); // TODO: change these and use proper logging
    println!("{}", board);
    // We unwrap the option because the key must exist, since we just created it
    ApiResponse::game_created(key, &board)
}

#[post("/new/from_fen", data = "<fen_data>")]
pub fn create_game_from_fen(fen_data: Json<FenData>, state: &StateMutex) -> ApiResponse {
    let mut inner_state = state.inner().lock().unwrap();
    match inner_state.create_game_from_fen(&fen_data.fen) {
        Ok(key) => {
            let board = inner_state.get_game(&key).unwrap().lock().unwrap(); // same as before
            println!("Creating a new custom game with key: {}", key); // TODO: change these and use proper logging
            println!("{}", board);
            ApiResponse::game_created(key, &board)
        },
        Err(msg) => ApiResponse::bad_request(msg),
    }
}