use shakmat_core::{Board, Move};
use super::messages::TurnInfo;

use rand::Rng;
use std::collections::HashMap;

const KEY_LENGTH: u32 = 15;

pub struct ServerState {
    games: HashMap<String, Board>,
    moves: HashMap<String, HashMap<String, Move>>
}

impl ServerState {
    pub fn new() -> Self {
        ServerState { games: HashMap::new(), moves: HashMap::new() }
    }

    pub fn create_game_default(&mut self) -> String {
        let key = random_string(KEY_LENGTH);
        self.games.insert(key.clone(), Board::default());
        self.update_game_moves(&key);
        key
    }

    
    pub fn create_game_from_fen(&mut self, fen: &str) -> Result<String, String> {
        let key = random_string(KEY_LENGTH);
        self.games.insert(key.clone(), Board::from_fen(fen)?);
        self.update_game_moves(&key);
        Ok(key)
    }
    
    pub fn get_turn_info(&self, key: &str) -> Option<TurnInfo> {
        self.games.get(key).map(TurnInfo::from_board)
    }
    
    pub fn get_board(&self, key: &str) -> Option<&Board> {
        self.games.get(key)
    }

    pub fn get_game_moves(&self, key: &str) -> Option<&HashMap<String, Move>> {
        self.moves.get(key)
    }
    
    // It is assumed that the move will always be legal, as the handler
    // will refuse to make it if it is not in the moves map for the board,
    // and that the key always exists
    pub fn make_move(&mut self, key: &str, movement: Move) -> Result<(), String> {
        match self.games[key].make_move(&movement, false) {
            Ok(new_board) => {
                println!("{}", new_board);
                self.games.insert(key.to_string(), new_board);
                self.update_game_moves(key);
                Ok(())
            },
            Err(msg) => Err(msg)
        }
    }

    // It is assumed that this will always be called with a key that exists
    fn update_game_moves(&mut self, game_key: &str) {
        let move_map = self.games[game_key].legal_moves()
            .into_iter()
            .map(move |mv| (mv.to_string().to_lowercase(), mv))
            .collect();

        self.moves.insert(game_key.to_string(), move_map);
    }
}

fn random_string(length: u32) -> String {
    let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789-_";
    let mut rng = rand::thread_rng();
    (0..length).map(|_| charset[rng.gen_range(0..charset.len())] as char).collect()
}