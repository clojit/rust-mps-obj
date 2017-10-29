//! Object formats

use std::sync::Arc;

use ffi::{mps_fmt_destroy, mps_fmt_t};
use arena::{Arena, ArenaRef};

pub mod area;

/// Generic MPS object format interface.
pub trait Format {
    fn as_raw(&self) -> mps_fmt_t;
}

/// Clone-able handle to a type-erased object format.
///
/// By holding on to this handle, the object format and arena it belongs to
/// will be kept alive.
#[derive(Clone)]
pub struct FormatRef {
    arena: ArenaRef,
    fmt: Arc<Format>,
}

impl FormatRef {
    fn new<F: Format + 'static>(arena: ArenaRef, fmt: F) -> Self {
        FormatRef {
            arena: arena,
            fmt: Arc::new(fmt),
        }
    }

    /// Access the arena this format belongs to
    pub fn arena(&self) -> &Arena {
        &self.arena
    }
}

impl Format for FormatRef {
    fn as_raw(&self) -> mps_fmt_t {
        self.fmt.as_raw()
    }
}

/// RAII-handle for a raw object format pointer.
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
