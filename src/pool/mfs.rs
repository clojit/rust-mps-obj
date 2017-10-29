//! Manual Fixed Size pool

use pool::{Pool, PoolRef};

pub struct MfsPool {
    size: usize,
    pool: PoolRef,
}
