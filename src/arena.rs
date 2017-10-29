use std::sync::Arc;
use std::ptr;

use errors::{Error, Result};
use ffi::{mps_arena_class_vm, mps_arena_committed, mps_arena_create_k, mps_arena_destroy,
          mps_arena_reserved, mps_arena_t};

pub trait Arena {
    fn as_raw(&self) -> mps_arena_t;

    fn commited(&self) -> usize {
        unsafe { mps_arena_committed(self.as_raw()) }
    }

    fn reserved(&self) -> usize {
        unsafe { mps_arena_reserved(self.as_raw()) }
    }
}

#[derive(Clone)]
pub struct ArenaRef {
    arena: Arc<Arena>,
}

impl ArenaRef {
    fn new<A: Arena + 'static>(arena: A) -> Self {
        ArenaRef {
            arena: Arc::new(arena),
        }
    }
}

impl Arena for ArenaRef {
    fn as_raw(&self) -> mps_arena_t {
        self.arena.as_raw()
    }
}

pub struct VmArena {
    inner: ArenaRef,
}

impl VmArena {
    pub fn with_capacity(capacity: usize) -> Result<Self> {
        let args = mps_args! {
             MPS_KEY_ARENA_SIZE: capacity,
        };

        let arena = unsafe {
            let mut arena: mps_arena_t = ptr::null_mut();
            let res = mps_arena_create_k(&mut arena, mps_arena_class_vm(), args);

            Error::result(res).map(|_| RawArena { arena })
        }?;

        Ok(VmArena {
            inner: ArenaRef::new(arena),
        })
    }
}

impl Arena for VmArena {
    fn as_raw(&self) -> mps_arena_t {
        self.inner.as_raw()
    }
}

impl Into<ArenaRef> for VmArena {
    fn into(self) -> ArenaRef {
        self.inner
    }
}

struct RawArena {
    arena: mps_arena_t,
}

impl Arena for RawArena {
    fn as_raw(&self) -> mps_arena_t {
        self.arena
    }
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
        let _ = VmArena::with_capacity(ARENA_TEST_SIZE).unwrap();
    }

    #[test]
    fn arena_commited() {
        let arena = VmArena::with_capacity(ARENA_TEST_SIZE).unwrap();
        assert!(arena.reserved() > arena.commited());
    }
}
