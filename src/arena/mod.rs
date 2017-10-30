//! Arena implementations

use std::sync::Arc;

use ffi::{mps_arena_committed, mps_arena_destroy, mps_arena_reserved, mps_arena_t};

pub mod vm;

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