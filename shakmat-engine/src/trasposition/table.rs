use std::mem::MaybeUninit;
use shakmat_core::Move;

use super::{TTEntry, TTData, NodeType};

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
    pub fn get_entry(&self, zobrist_key: u64, depth: u8, tt_move: &mut Option<Move>) -> Option<TTData> {
        let index = zobrist_key as usize % self.size;
        let entry = unsafe {
            (*self.ptr.add(index)).assume_init()
        };

        if entry.zobrist() != zobrist_key {
            return None;
        }

        // The entry key matches, load the best move regardless of depth
        let entry_data = unsafe { entry.data().assume_init() };
        *tt_move = entry_data.best_move;

        // If the stored depth is higher, use the stored data
        if entry_data.depth >= depth {
            Some(entry_data)
        } else {
            None
        }
    }

    // We only replace an entity if any of the following is true:
    // - The zobrist key is different
    // - The new depth is higher
    // - The stored entry has a different flag and it's not exact
    pub fn write_entry(&self, zobrist_key: u64, entry: TTEntry) {
        let index = zobrist_key as usize % self.size;
        let prev_entry = unsafe {
            (*self.ptr.add(index)).assume_init()
        };

        if prev_entry.zobrist() != zobrist_key {
            // The previous zobrist is different (or zero), overwrite the entry
            unsafe {
                *self.ptr.add(index) = MaybeUninit::new(entry);
            }
        } else {
            // The previous zobrist is the same, check if the new entry is better
            let prev_data = unsafe { prev_entry.data().assume_init() };
            let new_data = unsafe { entry.data().assume_init() };

            if new_data.depth > prev_data.depth || 
               (new_data.node_type() != prev_data.node_type() && prev_data.node_type() != NodeType::Exact) {
                    unsafe {
                        *self.ptr.add(index) = MaybeUninit::new(entry);
                    }
               }
        }
    }
}