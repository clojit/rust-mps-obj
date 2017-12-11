//! Virtual memory arena

use std::sync::Arc;
use std::ptr;

use errors::{Result, Error};
use ffi::{mps_arena_class_vm, mps_arena_create_k, mps_arena_t};
use arena::{Arena, ArenaRef, RawArena};

/// An MPS arena backed by virtual memory.
///
/// See [the reference](https://www.ravenbrook.com/project/mps/master/manual/html/topic/arena.html#virtual-memory-arenas)
/// for details.
#[derive(Clone)]
pub struct VmArena {
    raw: Arc<RawArena>,
}

impl VmArena {
    /// Creates a new virtual memory arena with the specified initial size
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
            raw: Arc::new(arena),
        })
    }
}

impl Arena for VmArena {
    fn as_raw(&self) -> mps_arena_t {
        self.raw.arena
    }
}

impl ArenaRef for VmArena {
    fn acquire(&self) -> Self {
        self.clone()
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
