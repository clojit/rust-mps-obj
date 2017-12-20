//! Manual Fixed Size pool

use std::sync::Arc;
use std::ptr;

use errors::{Result, Error};

use pool::{RawPool};
use arena::{ArenaRef, Arena};
use fmt::{FormatRef};
use ffi::{mps_pool_t, mps_pool_create_k, mps_class_amc, mps_ap_create_k};

#[derive(Clone)]
pub struct AmcPool<F> {
    raw: Arc<RawPool>,
    fmt: F
}

impl<F : FormatRef> AmcPool<F> {
    /// Creates a new virtual memory arena with the specified initial size
    pub fn with_format(fmt: F) -> Result<Self> {
        let args = mps_args! {
            MPS_KEY_FORMAT: fmt.as_raw(),
        };

        let pool = unsafe {
            let mut pool: mps_pool_t = ptr::null_mut();
            let res = mps_pool_create_k(&mut pool, fmt.arena().as_raw(), mps_class_amc(), args);

            Error::result(res).map(|_| RawPool { pool })
        }?;

        Ok(AmcPool {
            raw: Arc::new(pool),
            fmt: fmt.acquire()
        })
    }
}


#[cfg(test)]
mod tests {
    use arena::vm::VmArena;
    use fmt::area::AreaFormat;
    use fmt::area::ReferenceTag;
    use super::*;

    const ARENA_TEST_SIZE: usize = 2 << 32;

    #[test]
    fn simple_alloc() {
        let a = VmArena::with_capacity(ARENA_TEST_SIZE).unwrap();

        struct NanBox;

        impl ReferenceTag for NanBox {
            const MASK: u64 = 0xFFF0_0000_0000_0000;
            const PATTERN : u64 = 0;
        }

        let f = AreaFormat::tagged::<NanBox>(&a).unwrap();

        let amc = AmcPool::with_format(f).unwrap();

        let ap = unsafe {

            let mut ap: mps_ap_t = ptr::null_mut();

            let res = mps_pool_create_k(&mut ap, amc.raw.as_raw(), mps_args_none);

            //Error::result(res).map(|_| RawPool { pool })
        }?;





/*
        let pool = AmcPool::<i32, _>::with_arena(&a, &f).unwrap();

        let five = pool.alloc(|| 5).unwrap();
        assert_eq!(*five, 5);

        let six = pool.alloc(|| 6).unwrap();
        assert_eq!(*six, 6);*/
    }
}