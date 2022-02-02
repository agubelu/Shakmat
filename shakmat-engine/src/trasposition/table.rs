use std::mem::MaybeUninit;
use super::{TTEntry, TTData};

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

    pub fn get_entry(&self, zobrist_key: u64) -> Option<TTData> {
        let index = zobrist_key as usize % self.size;
        let entry = unsafe {
            (*self.ptr.add(index)).assume_init()
        };

        if entry.zobrist() == zobrist_key {
            unsafe { Some(entry.data().assume_init()) }
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