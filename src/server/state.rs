use crate::chess::{Board, Move};
use super::messages::TurnInfo;

use rand::Rng;
use std::collections::HashMap;

const KEY_LENGTH: u32 = 15;

pub struct ServerState {
    games: HashMap<String, Board>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState { games: HashMap::new() }
    }

    pub fn create_game_default(&mut self) -> String {
        let key = random_string(KEY_LENGTH);
        self.games.insert(key.clone(), Board::default());
        key
    }
    
    pub fn create_game_from_fen(&mut self, fen: &str) -> Result<String, String> {
        let key = random_string(KEY_LENGTH);
        self.games.insert(key.clone(), Board::from_fen(fen)?);
        Ok(key)
    }

    pub fn get_turn_info(&self, key: &str) -> Option<TurnInfo> {
        self.games.get(key).map(|board| TurnInfo::from_board(board))
    }

    // Returns None if the game with such key was not found
    // If it is found, the inner Result represents the result of the move
    pub fn make_move(&mut self, key: &str, movement: Move) -> Option<Result<(), String>> {
        match self.games.get(key) {
            None => None,
            Some(board) => {
                match board.make_move(movement, true) {
                    Ok(new_board) => {
                        println!("{}", new_board);
                        self.games.insert(key.to_string(), new_board);
                        Some(Ok(()))
                    },
                    Err(msg) => Some(Err(msg))
                }
            }
        }
    }
}

fn random_string(length: u32) -> String {
    let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789-_";
    let mut rng = rand::thread_rng();
    (0..length).map(|_| charset[rng.gen_range(0..charset.len())] as char).collect()
}