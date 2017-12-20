

pub mod amc_ap;

use ffi::{mps_ap_t};

pub trait AllocationPoint {
    fn as_raw(&self) -> mps_ap_t;
}

pub trait AllocationPointRef: AllocationPoint {
    fn acquire(&self) -> Self;
}

#[derive(Debug)]
struct RawAllocationPoint {
    ap: mps_ap_t,
}

impl Drop for RawAllocationPoint {
    fn drop(&mut self) {
        unsafe { mps_ap_destroy(self.ap) }
    }
}
