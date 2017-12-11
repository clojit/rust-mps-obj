//! Arena implementations

use ffi::{mps_arena_committed, mps_arena_destroy, mps_arena_reserved, mps_arena_t};

pub mod vm;

/// Generic MPS arena interface
pub trait Arena: Clone {
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


/// RAII-handle for a raw arena pointer. Destroys the underlying arena on drop.
struct RawArena {
    arena: mps_arena_t,
}

impl Drop for RawArena {
    fn drop(&mut self) {
        unsafe { mps_arena_destroy(self.arena) }
    }
}
