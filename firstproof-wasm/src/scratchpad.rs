use std::cell::RefCell;

thread_local! {
    static SCRATCHPAD: RefCell<ScratchpadBuffer> = RefCell::new(ScratchpadBuffer::new(256 * 1024 * 1024));
}

pub struct ScratchpadBuffer {
    buffer: Vec<u8>,
}

impl ScratchpadBuffer {
    pub fn new(initial_capacity: usize) -> Self {
        Self {
            buffer: vec![0u8; initial_capacity],
        }
    }

    pub fn resize(&mut self, new_capacity: usize) {
        if self.buffer.len() < new_capacity {
            self.buffer.resize(new_capacity, 0);
        }
    }

    pub fn ptr(&mut self) -> *mut u8 {
        self.buffer.as_mut_ptr()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn slice(&self, start: usize, len: usize) -> Option<&[u8]> {
        if start + len <= self.buffer.len() {
            Some(&self.buffer[start..start + len])
        } else {
            None
        }
    }

    pub fn slice_mut(&mut self, start: usize, len: usize) -> Option<&mut [u8]> {
        if start + len <= self.buffer.len() {
            Some(&mut self.buffer[start..start + len])
        } else {
            None
        }
    }
}

pub fn with_scratchpad<F, R>(f: F) -> R
where
    F: FnOnce(&mut ScratchpadBuffer) -> R,
{
    SCRATCHPAD.with(|cell| {
        let mut buf = cell.borrow_mut();
        f(&mut buf)
    })
}
