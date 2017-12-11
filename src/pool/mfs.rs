//! Manual Fixed Size pool

use std::sync::Arc;
use std::ptr;
use std::mem;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use errors::{Result, Error};

use pool::{Pool, PoolRef, RawPool};
use arena::{ArenaRef};

use ffi::{mps_addr_t, mps_alloc, mps_free, mps_pool_t, mps_pool_create_k, mps_class_mfs};

#[derive(Clone)]
pub struct MfsPool<T, A> {
    raw: Arc<RawPool>,
    arena: A,
    _marker: PhantomData<T>,
}

impl<T, A: ArenaRef> MfsPool<T, A> {
    /// Creates a new virtual memory arena with the specified initial size
    pub fn with_arena(arena: &A) -> Result<Self> {
        let args = mps_args! {
            MPS_KEY_MFS_UNIT_SIZE: mem::size_of::<T>(),
        };

        let pool = unsafe {
            let mut pool: mps_pool_t = ptr::null_mut();
            let res = mps_pool_create_k(&mut pool, arena.as_raw(), mps_class_mfs(), args);

            Error::result(res).map(|_| RawPool { pool })
        }?;

        Ok(MfsPool {
            raw: Arc::new(pool),
            arena: arena.acquire(),
            _marker: PhantomData,
        })
    }
}

impl<T, A> Pool for MfsPool<T, A> {
    fn as_raw(&self) -> mps_pool_t {
        self.raw.pool
    }
}

impl<T, A: ArenaRef> PoolRef for MfsPool<T, A> {
    type Arena = A;

    fn acquire(&self) -> Self {
        MfsPool {
            raw: self.raw.clone(),
            arena: self.arena.acquire(),
            _marker: PhantomData,
        }
    }
    
    fn arena(&self) -> &Self::Arena {
        &self.arena
    }
}

impl<T, A: ArenaRef> MfsPool<T, A> {
    pub fn alloc<F: FnOnce() -> T>(&self, initializer: F) -> Result<MfsBox<T, A>> {
        let addr = unsafe {
            let mut addr: mps_addr_t = ptr::null_mut();
            let size = mem::size_of::<T>();
            Error::result(mps_alloc(&mut addr, self.as_raw(), size))?;

            let base: *mut T = addr as *mut _;
            ptr::write(base, initializer());

            addr
        };

        Ok(MfsBox {
            addr: addr,
            pool: self.acquire(),
            _marker: PhantomData,
        })
    }
}


pub struct MfsBox<T, A> {
    addr: mps_addr_t,
    pool: MfsPool<T, A>,
    _marker: PhantomData<T>,
}

impl<T, A> Drop for MfsBox<T, A> {
    fn drop(&mut self) {
        unsafe {
            let base: *mut T = self.addr as *mut _;
            ptr::drop_in_place(base);

            let size = mem::size_of::<T>();
            mps_free(self.pool.as_raw(), self.addr, size)
        }
    }
}

impl<T, A> Deref for MfsBox<T, A> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            let addr: *const T = self.addr as *const _;
            addr.as_ref().unwrap()
        }
    }
}

impl<T, A> DerefMut for MfsBox<T, A> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            let addr: *mut T = self.addr as *mut _;
            addr.as_mut().unwrap()
        }
    }
}


#[cfg(test)]
mod tests {
    use arena::vm::VmArena;
    use super::*;

    const ARENA_TEST_SIZE: usize = 2 << 32;

    const HANDLE_TABLE: usize = 128;
    type TableType = [mps_addr_t; HANDLE_TABLE];

    #[test]
    fn alloc_handle_table() {
        let a = VmArena::with_capacity(ARENA_TEST_SIZE).unwrap();

        let pool = MfsPool::<TableType, _>::with_arena(&a).unwrap();

        // allocates a zeroed array of 128 mps_addr_ts
        let c1 = pool.alloc(|| unsafe { mem::zeroed() }).unwrap();

        assert_eq!(c1.len(), HANDLE_TABLE);

        assert!(pool.total_size() > 0 );

        println!("{}" , (pool.total_size() / 8) );
        println!("{}" ,  (pool.free_size()/8));
    }
}
