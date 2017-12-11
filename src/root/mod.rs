//! Roots and ranks

pub mod area;

use ffi::{mps_root_t};

/// Generic root interface
pub trait Root {
    fn as_raw(&self) ->  mps_root_t;
}
