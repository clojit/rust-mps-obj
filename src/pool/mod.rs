//! Memory pool implementation and interfaces

pub mod mfs;

use arena::{ArenaRef};
use ffi::{mps_pool_destroy, mps_pool_free_size, mps_pool_t, mps_pool_total_size};

/// Generic pool interface
pub trait Pool {
    fn as_raw(&self) -> mps_pool_t;

    /// Returns the total size the pool occupies in its arena
    fn total_size(&self) -> usize {
        unsafe { mps_pool_total_size(self.as_raw()) }
    }

    /// Amount of memory currently unallocated but assigned to this pool
    fn free_size(&self) -> usize {
        unsafe { mps_pool_free_size(self.as_raw()) }
    }
}

pub trait PoolRef: Pool {
    type Arena: ArenaRef;

    fn acquire(&self) -> Self;
    fn arena(&self) -> &Self::Arena;
}

/// RAII-style handle
#[derive(Debug)]
struct RawPool {
    pool: mps_pool_t,
}

impl Pool for RawPool {
    fn as_raw(&self) -> mps_pool_t {
        self.pool
    }
}

impl Drop for RawPool {
    fn drop(&mut self) {
        unsafe { mps_pool_destroy(self.pool) }
    }
}
