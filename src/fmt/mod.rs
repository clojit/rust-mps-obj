//! Object formats

use ffi::{mps_fmt_destroy, mps_fmt_t};
use arena::{ArenaRef};

pub mod area;

/// Generic MPS object format interface.
pub trait Format {
    fn as_raw(&self) -> mps_fmt_t;
}

pub trait FormatRef: Format {
    type Arena: ArenaRef;

    fn acquire(&self) -> Self;
    fn arena(&self) -> &Self::Arena;
}

/// RAII-handle for a raw object format pointer.
#[derive(Debug)]
struct RawFormat {
    fmt: mps_fmt_t,
}

impl Format for RawFormat {
    fn as_raw(&self) -> mps_fmt_t {
        self.fmt
    }
}

impl Drop for RawFormat {
    fn drop(&mut self) {
        unsafe {
            mps_fmt_destroy(self.fmt);
        }
    }
}
