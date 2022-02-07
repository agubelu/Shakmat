use shakmat_core::{Board, Move, DEFAULT_FEN};
use super::messages::TurnInfo;

use rand::Rng;
use std::collections::HashMap;

const KEY_LENGTH: u32 = 15;

pub struct ServerState {
    games: HashMap<String, GameData>,
}

struct GameData {
    pub board: Board,
    pub current_moves: HashMap<String, Move>,
    pub previous_positions: Vec<u64>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState { games: HashMap::new() }
    }

    pub fn create_game_default(&mut self) -> String {
        let key = random_string(KEY_LENGTH);
        self.games.insert(key.clone(), GameData::default());
        self.update_game_moves(&key);
        key
    }

    
    pub fn create_game_from_fen(&mut self, fen: &str) -> Result<String, String> {
        let key = random_string(KEY_LENGTH);
        self.games.insert(key.clone(), GameData::from_fen(fen)?);
        self.update_game_moves(&key);
        Ok(key)
    }
    
    pub fn get_turn_info(&self, key: &str) -> Option<TurnInfo> {
        self.games.get(key).map(|gd| TurnInfo::from_board(&gd.board, &gd.previous_positions))
    }
    
    pub fn get_board(&self, key: &str) -> Option<&Board> {
        self.games.get(key).map(|gd| &gd.board)
    }

    pub fn get_game_moves(&self, key: &str) -> Option<&HashMap<String, Move>> {
        self.games.get(key).map(|gd| &gd.current_moves)
    }
    
    pub fn get_history(&self, key: &str) -> Option<&Vec<u64>> {
        self.games.get(key).map(|gd| &gd.previous_positions)
    }
    
    // It is assumed that the move will always be legal, as the handler
    // will refuse to make it if it is not in the moves map for the board,
    // and that the key always exists
    pub fn make_move(&mut self, key: &str, movement: Move) -> Result<(), String> {
        self.games[key].board.make_move(&movement, false)
            .map(move |new_board| {
                println!("{}", new_board);
                self.get_game_mut(key).board = new_board;
                self.get_game_mut(key).previous_positions.push(new_board.zobrist_key());
                self.update_game_moves(key);
            })
    }

    // It is assumed that this will always be called with a key that exists
    fn update_game_moves(&mut self, game_key: &str) {
        let move_map = self.games[game_key].board.legal_moves()
            .into_iter()
            .map(move |mv| (mv.to_string().to_lowercase(), mv))
            .collect();

        self.get_game_mut(game_key).current_moves = move_map;
    }

    // Mutably gets the GameData entry associated to a key that is assumed to exist
    fn get_game_mut(&mut self, key: &str) -> &mut GameData {
        self.games.get_mut(key).unwrap()
    }
}

impl GameData {
    fn from_fen(fen: &str) -> Result<Self, String> {
        let board = Board::from_fen(fen)?;
        let mut previous_positions = Vec::with_capacity(250);
        previous_positions.push(board.zobrist_key());

        // ServerState.update_game_moves() must be called after creation to fill this!
        let current_moves = HashMap::new();
        Ok(Self { board, current_moves, previous_positions})
    }
}

impl Default for GameData {
    fn default() -> Self {
        Self::from_fen(DEFAULT_FEN).unwrap()
    }
}

fn random_string(length: u32) -> String {
    let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();
    (0..length).map(|_| charset[rng.gen_range(0..charset.len())] as char).collect()
}