use std::cell::RefCell;

/// Pre-allocated reusable scratch arena for RAW decode and demosaic buffers.
/// Eliminates 600MB+ per-photo heap allocation churn by retaining capacities
/// across successive photo decodes on the same thread.
pub struct DecodeScratchArena {
    pub rgb_raw_bytes: Vec<u8>,
    pub linear_rgb_f32: Vec<f32>,
}

impl DecodeScratchArena {
    pub fn new() -> Self {
        Self {
            rgb_raw_bytes: Vec::new(),
            linear_rgb_f32: Vec::new(),
        }
    }

    /// Resizes `rgb_raw_bytes` to at least `min_bytes` without shrinking.
    pub fn acquire_raw_bytes(&mut self, min_bytes: usize) -> &mut [u8] {
        if self.rgb_raw_bytes.len() < min_bytes {
            self.rgb_raw_bytes.resize(min_bytes, 0);
        } else {
            self.rgb_raw_bytes[..min_bytes].fill(0);
        }
        &mut self.rgb_raw_bytes[..min_bytes]
    }

    /// Resizes `linear_rgb_f32` to at least `min_floats` without shrinking.
    pub fn acquire_linear_f32(&mut self, min_floats: usize) -> &mut [f32] {
        if self.linear_rgb_f32.len() < min_floats {
            self.linear_rgb_f32.resize(min_floats, 0.0);
        } else {
            self.linear_rgb_f32[..min_floats].fill(0.0);
        }
        &mut self.linear_rgb_f32[..min_floats]
    }

    /// Returns the currently allocated memory in bytes.
    pub fn capacity_bytes(&self) -> usize {
        self.rgb_raw_bytes.capacity() + (self.linear_rgb_f32.capacity() * std::mem::size_of::<f32>())
    }
}

thread_local! {
    static THREAD_DECODE_ARENA: RefCell<DecodeScratchArena> = RefCell::new(DecodeScratchArena::new());
}

/// Executes a closure with access to the thread-local `DecodeScratchArena`.
pub fn with_decode_arena<R, F: FnOnce(&mut DecodeScratchArena) -> R>(f: F) -> R {
    THREAD_DECODE_ARENA.with(|arena| f(&mut arena.borrow_mut()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scratch_arena_retains_capacity() {
        with_decode_arena(|arena| {
            let b1 = arena.acquire_raw_bytes(1024);
            assert_eq!(b1.len(), 1024);
            let cap_after_1 = arena.rgb_raw_bytes.capacity();
            assert!(cap_after_1 >= 1024);

            // Request smaller size: capacity must NOT shrink
            let b2 = arena.acquire_raw_bytes(512);
            assert_eq!(b2.len(), 512);
            assert_eq!(arena.rgb_raw_bytes.capacity(), cap_after_1);

            let f1 = arena.acquire_linear_f32(2048);
            assert_eq!(f1.len(), 2048);
            let f_cap_1 = arena.linear_rgb_f32.capacity();
            assert!(f_cap_1 >= 2048);

            let f2 = arena.acquire_linear_f32(100);
            assert_eq!(f2.len(), 100);
            assert_eq!(arena.linear_rgb_f32.capacity(), f_cap_1);
        });
    }
}
