//! Arena implementations

use std::sync::Arc;
use std::ptr;

use errors::{Error, Result};
use ffi::{mps_arena_class_vm, mps_arena_committed, mps_arena_create_k, mps_arena_destroy,
          mps_arena_reserved, mps_arena_t};

/// Generic MPS arena interface
pub trait Arena {
    /// Returns a raw pointer to the underlying MPS arena.
    ///
    /// Note that this pointer must never outlive self.
    fn as_raw(&self) -> mps_arena_t;

    /// Return the total committed memory for an arena.
    ///
    /// See: [`mps_arena_committed`](https://www.ravenbrook.com/project/mps/master/manual/html/topic/arena.html#c.mps_arena_committed)
    fn commited(&self) -> usize {
        unsafe { mps_arena_committed(self.as_raw()) }
    }

    /// Return the total address space reserved by an arena, in bytes.
    ///
    /// See: [`mps_arena_reserved`](https://www.ravenbrook.com/project/mps/master/manual/html/topic/arena.html#c.mps_arena_reserved)
    fn reserved(&self) -> usize {
        unsafe { mps_arena_reserved(self.as_raw()) }
    }
}

/// Clone-able handle to a type-erased arena.
///
/// The underlying arena is kept alive as long as there are `ArenaRef`s
/// holding on to it. Must be used when constructing object formats, pool
/// or other resources which must not outlive the arena.
#[derive(Clone)]
pub struct ArenaRef {
    arena: Arc<Arena>,
}

impl ArenaRef {
    /// Construct a initial reference for the given arena.
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

/// An MPS arena backed by virtual memory.
///
/// See [the reference](https://www.ravenbrook.com/project/mps/master/manual/html/topic/arena.html#virtual-memory-arenas)
/// for details.
pub struct VmArena {
    inner: ArenaRef,
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

/// RAII-handle for a raw arena pointer. Destroys the underlying arena on drop.
struct RawArena {
    arena: mps_arena_t,
}

/// This type impelents the `Arena` such that it can be wrapped inside of
/// a ArenaRef.
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
