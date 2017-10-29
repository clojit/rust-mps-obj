//! Thread registration

use std::ptr;

use ffi::{mps_thr_t, mps_thread_reg, mps_thread_dereg};
use arena::{Arena, ArenaRef};
use errors::{Result, Error};

/// Registered thread, holds on to arena
pub struct Thread {
    thr: mps_thr_t,
    arena: ArenaRef,
}

impl Thread {
    /// Registers the current thread with the specified arena
    pub fn register<A: Into<ArenaRef>>(arena: A) -> Result<Self> {
        unsafe {
            let arena = arena.into();
            let mut thr = ptr::null_mut();
            let res = mps_thread_reg(&mut thr, arena.as_raw());
            Error::result(res).map(|_| Thread {
                thr, arena
            })
        }
    }

    /// Return a the raw thread pointer
    pub fn as_raw(&self) -> mps_thr_t {
        self.thr
    }

    /// Access the arena this thread is registered in
    pub fn arena(&self) -> &Arena {
        &self.arena
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        unsafe {
            mps_thread_dereg(self.thr)
        }
    }
}
