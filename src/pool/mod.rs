//! Memory pool implementation and interfaces

pub mod mfs;

use std::sync::Arc;
use std::ptr;
use std::slice;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;

use arena::{Arena};
use errors::{Error, Result};
use ffi::{mps_addr_t, mps_alloc, mps_free, mps_pool_destroy, mps_pool_free_size, mps_pool_t, mps_pool_total_size};

/// Generic pool interface
pub trait Pool: Clone {
    type Arena: Arena;

    fn as_raw(&self) -> mps_pool_t;
    fn arena(&self) -> &Self::Arena;

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
pub struct Chunk<T, P: ManualAllocPool> {
    pool: P,
    addr: mps_addr_t,
    len: usize,
    _marker: PhantomData<Vec<T>>,
}

pub trait ManualAllocPool: Pool + Sized {
    fn alloc<T: Default>(&self, len: usize) -> Result<Chunk<T, Self>> {
        // TODO(gandro): check len fits in isize and is nonzero
        let pool = self.clone();
        let addr = unsafe {
            // allocate
            let mut addr: mps_addr_t = ptr::null_mut();
            let size = len * mem::size_of::<T>();
            Error::result(mps_alloc(&mut addr, pool.as_raw(), size))?;

            // initialize with default value
            let base: *mut T = addr as *mut _;
            for i in 0..len as isize {
                ptr::write(base.offset(i), Default::default());
            }

            addr
        };

        Ok(Chunk {
            pool: self.clone().into(),
            addr: addr,
            len: len,
            _marker: PhantomData,
        })
    }
}

impl<T, P: ManualAllocPool> Drop for Chunk<T, P> {
    fn drop(&mut self) {
        unsafe {
            let base: *mut T = self.addr as *mut _;
            for i in 0..self.len as isize {
                ptr::drop_in_place(base.offset(i));
            }

            let size = self.len * mem::size_of::<T>();
            mps_free(self.pool.as_raw(), self.addr, size)
        }
    }
}

impl<T, P: ManualAllocPool> Deref for Chunk<T, P> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe {
            let addr: *const T = self.addr as *const _;
            slice::from_raw_parts(addr, self.len)
        }
    }
}

impl<T, P: ManualAllocPool> DerefMut for Chunk<T, P> {
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
