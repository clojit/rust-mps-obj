use std::sync::Arc;
use std::ptr;

use errors::{Error, Result};
use ffi::{mps_arena_committed, mps_arena_destroy, mps_arena_reserved, mps_arena_t,
          rust_mps_create_vm_area};

#[derive(Clone)]
pub struct Arena {
    inner: Arc<RawArena>,
}

impl Arena {
    pub fn with_capacity(capacity: usize) -> Result<Self> {
        let mut arena: mps_arena_t = ptr::null_mut();
        let res = unsafe { rust_mps_create_vm_area(&mut arena, capacity) };

        Error::result(res).map(|_| {
            Arena {
                inner: Arc::new(RawArena { arena }),
            }
        })
    }

    fn commited(&self) -> usize {
        unsafe { mps_arena_committed(self.inner.arena) }
    }

    fn reserved(&self) -> usize {
        unsafe { mps_arena_reserved(self.inner.arena) }
    }
}

struct RawArena {
    arena: mps_arena_t,
}

impl Drop for RawArena {
    fn drop(&mut self) {
        unsafe { mps_arena_destroy(self.arena) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARENA_TEST_SIZE: usize = 2 << 32;

    #[test]
    fn arena_create_and_drop() {
        let _ = Arena::with_capacity(ARENA_TEST_SIZE).unwrap();
    }

    #[test]
    fn arena_commited() {
        let arena = Arena::with_capacity(ARENA_TEST_SIZE).unwrap();
        assert!(arena.reserved() > arena.commited());
    }
}
