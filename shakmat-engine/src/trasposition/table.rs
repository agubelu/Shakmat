use std::mem::MaybeUninit;
use shakmat_core::Move;

use crate::evaluation::Evaluation;

use super::{TTEntry, NodeType};

// Operations with the trasposition table are unsafe, as it is intended for
// lock-less multithreaded use, and data races will occur. It is up to us
// to detect when they do, and act accordingly.
pub struct TTable {
    size: usize,
    _content: Vec<MaybeUninit<TTEntry>>,
    ptr: *mut MaybeUninit<TTEntry>
}

impl TTable {
    pub fn new(size: usize) -> Self {
        let mut vec = Vec::with_capacity(size);
        unsafe {
            vec.set_len(size);
        }
        Self { ptr: vec.as_mut_ptr(), _content: vec, size }
    }

    // Returns a data entry from the table, if all of the following are true:
    // - The entry exists, and the zobrist key matches
    // - The depth of the search that stored the entry is at least that of
    //   the search that is querying for the entry, to avoid using info from
    //   shallower depths
    // - The score is in the appropriate bounds, depending on the type of entry
    pub fn get_entry(&self, zobrist_key: u64, depth: u8, alpha: Evaluation, beta: Evaluation, tt_move: &mut Option<Move>) -> Option<Evaluation> {
        let index = zobrist_key as usize % self.size;
        let entry = unsafe {
            (*self.ptr.add(index)).assume_init()
        };

        if entry.zobrist() == zobrist_key {
            let entry_data = unsafe { entry.data().assume_init() };
            *tt_move = entry_data.best_move;
            return None;

            if entry_data.depth >= depth {
                let node_type = entry_data.node_type();
                let score = entry_data.eval_score();
                match node_type {
                    NodeType::Exact if score >= alpha && score <= beta => Some(score),
                    NodeType::AlphaCutoff if score <= alpha => Some(alpha),
                    NodeType::BetaCutoff if score >= beta => Some(beta),
                    _ => None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn write_entry(&self, zobrist_key: u64, entry: TTEntry) {
        let index = zobrist_key as usize % self.size;
        unsafe {
            *self.ptr.add(index) = MaybeUninit::new(entry);
        }
    }
}