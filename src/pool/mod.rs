//! Memory pool implementation and interfaces

pub mod mfs;

use std::sync::Arc;
use std::ptr;
use std::slice;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;

use arena::{Arena, ArenaRef};
use errors::{Error, Result};
use ffi::{mps_addr_t, mps_alloc, mps_free, mps_pool_destroy, mps_pool_free_size, mps_pool_t, mps_pool_total_size};

/// Clone-able handle to a type-erased object pool.
#[derive(Clone)]
pub struct PoolRef {
    arena: ArenaRef,
    pool: Arc<Pool>,
}

impl PoolRef {
    fn new<P: Pool + 'static>(arena: ArenaRef, pool: P) -> Self {
        PoolRef {
            arena: arena,
            pool: Arc::new(pool),
        }
    }

    /// Access the arena this format belongs to
    pub fn arena(&self) -> &Arena {
        &self.arena
    }
}

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

/// A manually allocated chunk of fixed-size, homogenous memory
///
/// Will be freed on drop
pub struct Chunk<'pool, T: 'pool> {
    pool: mps_pool_t,
    addr: mps_addr_t,
    len: usize,
    _marker: PhantomData<&'pool mut T>,
}

pub trait ManualAllocPool: Pool {
    fn alloc<'pool, T: Default>(&'pool self, len: usize) -> Result<Chunk<'pool, T>> {
        // TODO(gandro): check len fits in isize and is nonzero
        let pool = self.as_raw();
        let addr = unsafe {
            // allocate
            let mut addr: mps_addr_t = ptr::null_mut();
            let size = len * mem::size_of::<T>();
            Error::result(mps_alloc(&mut addr, pool, size))?;

            // initialize with default value
            let base: *mut T = addr as *mut _;
            for i in 0..len as isize {
                ptr::write(base.offset(i), Default::default());
            }

            addr
        };

        Ok(Chunk {
            pool: pool,
            addr: addr,
            len: len,
            _marker: PhantomData,
        })
    }
}

impl<'pool, T> Drop for Chunk<'pool, T> {
    fn drop(&mut self) {
        unsafe {
            let base: *mut T = self.addr as *mut _;
            for i in 0..self.len as isize {
                ptr::drop_in_place(base.offset(i));
            }

            let size = self.len * mem::size_of::<T>();
            mps_free(self.pool, self.addr, size)
        }
    }
}

impl<'pool, T> Deref for Chunk<'pool, T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe {
            let addr: *const T = self.addr as *const _;
            slice::from_raw_parts(addr, self.len)
        }
    }
}

impl<'pool, T> DerefMut for Chunk<'pool, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            let addr: *mut T = self.addr as *mut _;
            slice::from_raw_parts_mut(addr, self.len)
        }
    }
}



/// RAII-style handle
struct RawPool {
    pool: mps_pool_t,
}

impl Drop for RawPool {
    fn drop(&mut self) {
        unsafe { mps_pool_destroy(self.pool) }
    }
}
