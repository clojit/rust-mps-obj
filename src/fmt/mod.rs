//! Object formats

use ffi::{mps_fmt_destroy, mps_fmt_t};
use arena::{Arena};

pub mod area;

/// Generic MPS object format interface.
pub trait Format {
    type Arena: Arena;

    fn as_raw(&self) -> mps_fmt_t;
    fn arena(&self) -> &Self::Arena;
}

/// RAII-handle for a raw object format pointer.
struct RawFormat {
    fmt: mps_fmt_t,
}

impl Drop for RawFormat {
    fn drop(&mut self) {
        unsafe {
            mps_fmt_destroy(self.fmt);
        }
    }
}
