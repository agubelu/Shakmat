use crate::chess::Board;

use rand::Rng;
use std::{collections::HashMap, sync::Mutex};

const KEY_LENGTH: u32 = 10;

pub struct ServerState {
    games: HashMap<String, Mutex<Board>>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState { games: HashMap::new() }
    }

    pub fn create_game_default(&mut self) -> String {
        let key = random_string(KEY_LENGTH);
        self.games.insert(key.clone(), Mutex::from(Board::default()));
        key
    }
    
    pub fn create_game_from_fen(&mut self, fen: &str) -> Result<String, String> {
        let key = random_string(KEY_LENGTH);
        self.games.insert(key.clone(), Mutex::from(Board::from_fen(fen)?));
        Ok(key)
    }

    pub fn get_game(&self, key: &str) -> Option<&Mutex<Board>> {
        self.games.get(key)
    }
}

fn random_string(length: u32) -> String {
    let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789-_";
    let mut rng = rand::thread_rng();
    (0..length).map(|_| charset[rng.gen_range(0..charset.len())] as char).collect()
}