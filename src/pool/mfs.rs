//! Manual Fixed Size pool

use std::sync::Arc;
use std::ptr;

use errors::{Result, Error};

use pool::{Pool, RawPool, ManualAllocPool, Chunk};
use arena::{Arena};
use arena::vm::{VmArena};

use ffi::{mps_pool_t, mps_pool_create_k, mps_class_mfs, mps_addr_t};

#[derive(Clone)]
pub struct MfsPool<A> {
    raw: Arc<RawPool>,
    arena: A,
}

impl<A: Arena> MfsPool<A> {
    /// Creates a new virtual memory arena with the specified initial size
    pub fn with_arena(arena: &A, unit_size : usize) -> Result<Self> {
        let args = mps_args! {
            MPS_KEY_MFS_UNIT_SIZE: unit_size,
        };

        let pool = unsafe {
            let mut pool: mps_pool_t = ptr::null_mut();
            let res = mps_pool_create_k(&mut pool, arena.as_raw(), mps_class_mfs(), args);

            Error::result(res).map(|_| RawPool { pool })
        }?;

        Ok(MfsPool {
            raw: Arc::new(pool),
            arena: arena.clone(),
        })
    }
}

impl<A: Arena> Pool for MfsPool<A> {
    type Arena = A;

    fn as_raw(&self) -> mps_pool_t {
        self.raw.pool
    }
    
    fn arena(&self) -> &Self::Arena {
        &self.arena
    }
}

impl<A: Arena> ManualAllocPool for MfsPool<A> {

}

#[cfg(test)]
mod tests {
    use super::*;

    const ARENA_TEST_SIZE: usize = 2 << 32;
    const HANDLE_TABLE: usize = (8 * 64);

    #[test]
    fn arena_create_and_drop() {
        let a = VmArena::with_capacity(ARENA_TEST_SIZE).unwrap();

        let pool = MfsPool::with_arena(&a, HANDLE_TABLE*8).unwrap();

        let c1 : Chunk<u64, _> = pool.alloc(HANDLE_TABLE).unwrap();

        assert_eq!(c1.len(), HANDLE_TABLE);

        assert!(pool.total_size() > 0 );

        println!("{}" , (pool.total_size() / 8) );
        println!("{}" ,  (pool.free_size()/8));
        //assert_eq!(pool, c1.pool);
    }
}
