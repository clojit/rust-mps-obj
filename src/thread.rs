//! Thread registration

use std::sync::Arc;
use std::ptr;

use ffi::{mps_thr_t, mps_thread_reg, mps_thread_dereg};
use arena::{Arena};
use errors::{Result, Error};

/// Registered thread, holds on to arena
pub struct Thread<A> {
    thr: mps_thr_t,
    arena: A,
}

impl<A: Arena> Thread<A> {
    /// Registers the current thread with the specified arena
    pub fn register(arena: &A) -> Result<Self> {
        unsafe {
            let arena = arena.clone();
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
    pub fn arena(&self) -> &A {
        &self.arena
    }
}

impl<A> Drop for Thread<A> {
    fn drop(&mut self) {
        unsafe {
            mps_thread_dereg(self.thr)
        }
    }
}
