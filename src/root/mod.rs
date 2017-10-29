//! Roots and ranks

pub mod area;

use ffi::{mps_root_t, mps_root_destroy};

/// Generic root interface
pub trait Root {
    fn as_raw(&self) ->  mps_root_t;
}

/// RAII-style pointer wrapper
struct RawRoot {
    root: mps_root_t,
}

impl Drop for RawRoot {
    fn drop(&mut self) {
        unsafe {
            mps_root_destroy(self.root)
        }
    }
}
