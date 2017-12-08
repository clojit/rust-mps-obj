//! Manual Fixed Size pool

use std::ptr;

use errors::{Result, Error};

use pool::{Pool, PoolRef, RawPool};
use arena::vm::{VmArena};


use ffi::{mps_pool_t, mps_pool_create_k, mps_class_mfs};

pub struct MfsPool {
    pool: PoolRef
}

impl MfsPool {
    /// Creates a new virtual memory arena with the specified initial size
    pub fn with_arena(arena : VmArena, unit_size : usize) -> Result<Self> {
        let args = mps_args! {
            // MPS_KEY_MFS_UNIT_SIZE: unit_size,
        };

        let pool = unsafe {
            let mut pool: mps_pool_t = ptr::null_mut();
            let res = mps_pool_create_k(&mut pool, arena.as_raw(), mps_class_mfs(), args);

            Error::result(res).map(|_| RawPool { pool })
        }?;

        Ok(MfsPool {
            pool: PoolRef::new(arena.inner,pool)
        })
    }
}

impl Pool for MfsPool {
    fn as_raw(&self) -> mps_pool_t {
        self.pool.as_raw()
    }
}

impl Into<PoolRef> for MfsPool {
    fn into(self) -> PoolRef {
        self.pool
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const ARENA_TEST_SIZE: usize = 2 << 32;

    #[test]
    fn arena_create_and_drop() {
        let a = VmArena::with_capacity(ARENA_TEST_SIZE).unwrap();

        let pool = MfsPool::with_arena(a, 4);

        match pool {
            Ok(p) => {
                println!("Total Size: {:?}", p.total_size());
                println!("Free Size: {:?}", p.free_size());
            },
            Err(err) => println!("Error: {:?}", err),
        }


    }


}
