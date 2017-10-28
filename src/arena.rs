use std::sync::Arc;
use std::ptr;

use errors::{Error, Result};
use ffi::{mps_arena_class_vm, mps_arena_committed, mps_arena_create_k, mps_arena_destroy,
          mps_arena_reserved, mps_arena_t};

#[derive(Clone)]
pub struct Arena {
    ptr: Arc<RawArena>,
}

impl Arena {
    pub unsafe fn from_raw(arena: mps_arena_t) -> Self {
        Arena {
            ptr: Arc::new(RawArena { arena }),
        }
    }

    pub unsafe fn as_raw_ptr(&self) -> mps_arena_t {
        self.ptr.arena
    }

    pub fn commited(&self) -> usize {
        unsafe { mps_arena_committed(self.ptr.arena) }
    }

    pub fn reserved(&self) -> usize {
        unsafe { mps_arena_reserved(self.ptr.arena) }
    }
}

#[derive(Clone)]
pub struct VmArena {
    arena: Arena,
}

impl VmArena {
    pub fn with_capacity(capacity: usize) -> Result<Self> {
        let args = mps_args! {
             MPS_KEY_ARENA_SIZE: capacity,
        };

        unsafe {
            let mut arena: mps_arena_t = ptr::null_mut();
            let res = mps_arena_create_k(&mut arena, mps_arena_class_vm(), args);

            Error::result(res).map(|_| {
                VmArena {
                    arena: Arena::from_raw(arena),
                }
            })
        }
    }
}

impl AsRef<Arena> for VmArena {
    fn as_ref(&self) -> &Arena {
        &self.arena
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
        let _ = VmArena::with_capacity(ARENA_TEST_SIZE).unwrap();
    }

    #[test]
    fn arena_commited() {
        let arena = VmArena::with_capacity(ARENA_TEST_SIZE).unwrap();
        assert!(arena.reserved() > arena.commited());
    }
}
