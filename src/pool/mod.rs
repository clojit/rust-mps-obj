//! Memory pool implementation and interfaces

use errors::{Error, Result};
use ffi::{mps_pool_destroy, mps_pool_free_size, mps_pool_t, mps_pool_total_size};

pub trait Pool {
    fn as_raw(&self) -> mps_pool_t;

    fn total_size(&self) -> usize {
        unsafe { mps_pool_total_size(self.as_raw()) }
    }

    fn free_size(&self) -> usize {
        unsafe { mps_pool_free_size(self.as_raw()) }
    }
}

struct RawPool {
    pool: mps_pool_t,
}

impl Drop for RawPool {
    fn drop(&mut self) {
        unsafe { mps_pool_destroy(self.pool) }
    }
}
