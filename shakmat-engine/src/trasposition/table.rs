use std::mem::MaybeUninit;
use super::TTEntry;

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

    pub unsafe fn get_entry(&self, key: u64) -> &MaybeUninit<TTEntry> {
        let index = key as usize % self.size;
        &*self.ptr.add(index)
    }

    pub unsafe fn write_entry(&self, key: u64, entry: TTEntry) {
        let index = key as usize % self.size;
        *self.ptr.add(index) = MaybeUninit::new(entry);
    }
}