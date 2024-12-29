use dashmap::DashMap;
use rayon::prelude::*;
use crate::{Board, Move};

type PerftCache = DashMap<(u64, usize), u64>;

impl Board {
    pub fn perft(&self, depth: usize) -> u64 {
        self._perft(depth, true, &DashMap::new())
    }

    pub fn perft_with_cache(&self, depth: usize, cache: &PerftCache) -> u64 {
        self._perft(depth, true, cache)
    }

    fn _perft(&self, depth: usize, multithread: bool, cache: &PerftCache) -> u64 {
        let key = self.zobrist_key();
        if let Some(res) = cache.get(&(key, depth)) {
            return *res;
        } else if depth == 1 {
            return self.legal_moves().len() as u64
        }

        let pseudo_moves = self.pseudolegal_moves();

        let res = if multithread {
            pseudo_moves.into_par_iter().filter_map(|mv| {
                let new_board = self.make_move(&mv);
                if matches!(mv, Move::LongCastle | Move::ShortCastle) || !new_board.is_check(self.turn_color()) {
                    Some(new_board._perft(depth - 1, false, cache))
                } else {
                    None
                }
            }).sum()
        } else {
            pseudo_moves.into_iter().filter_map(|mv| {
                let new_board = self.make_move(&mv);
                if matches!(mv, Move::LongCastle | Move::ShortCastle) || !new_board.is_check(self.turn_color()) {
                    Some(new_board._perft(depth - 1, false, cache))
                } else {
                    None
                }
            }).sum()
        };
        cache.insert((key, depth), res);
        res
    }
}