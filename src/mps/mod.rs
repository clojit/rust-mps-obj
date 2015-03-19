extern crate libc;

use std::rt::at_exit;
use std::mem;
use std::ptr;
use std::slice;
use std::marker;
use std::ops::{Deref, DerefMut};
use std::sync::{Once, ONCE_INIT};
use std::rt::heap::{allocate, deallocate};
use self::ffi::*;

pub mod ffi;

const MPS_HEAP_SIZE: usize = 32*1024*1024;

pub const MPS_HEADER_SIZE: usize = 8;
pub const MPS_WORD_SIZE: usize = 8;

#[repr(packed, C)]
pub struct Header {
    pub mps_type: u8,
    padding: u8,
    pub obj_type: u16,
    pub size: u32,
}

impl Header {
    pub fn len(&self) -> usize {
        self.size as usize / MPS_WORD_SIZE - 1
    }

    pub fn offset(&mut self, index: u16) -> *mut NanBox {
        assert!((index as usize) < self.len());
        let obj: *mut NanBox = unsafe { mem::transmute(self) };
        unsafe { obj.offset(1 + (index as isize)) }
    }
}

fn arena() -> mps_arena_t {
    static mut arena: mps_arena_t = 0 as mps_arena_t;
    static INIT: Once = ONCE_INIT;
    INIT.call_once(|| unsafe {
        let arenasize = MPS_HEAP_SIZE as libc::size_t;
        let res = rust_mps_create_vm_area(&mut arena, arenasize);
        assert!(res == 0);

        at_exit(|| unsafe {
            mps_arena_destroy(arena);
        });
    });

    unsafe { arena }
}

fn amc_pool() -> (mps_pool_t, mps_fmt_t) {
    static mut amc: mps_pool_t = 0 as mps_pool_t;
    static mut fmt: mps_fmt_t = 0 as mps_fmt_t;
    static INIT: Once = ONCE_INIT;
    INIT.call_once(|| unsafe {
        let res = rust_mps_create_amc_pool(&mut amc, &mut fmt, arena());
        assert!(res == 0);

        at_exit(|| unsafe {
            mps_pool_destroy(amc);
            mps_fmt_destroy(fmt);
        });
    });

    unsafe { (amc, fmt) }
}

pub fn gc() {
    let arena = arena();
    unsafe {
        mps_arena_collect(arena);
        mps_arena_release(arena);
    }
}

pub fn debug_printwalk() {
    let arena = arena();
    let (_, fmt) = amc_pool();
    unsafe {
        rust_mps_debug_print_reachable(arena, fmt);
    }
}

struct Thread(mps_thr_t);

thread_local! {
    static MPS_THREAD: Thread = unsafe {
        let mut thread = ptr::null_mut();
        let res = mps_thread_reg(&mut thread, arena());
        assert!(res == 0);
        Thread(thread)
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        let Thread(ref thread) = *self;
        unsafe {
            mps_thread_dereg(*thread);
        }
    }
}

struct AllocPoint(mps_ap_t);

thread_local! {
    static MPS_AMC_AP: AllocPoint = unsafe {
        let mut ap = ptr::null_mut();
        let (amc, _) = amc_pool();
        let res = rust_mps_create_ap(&mut ap, amc);
        assert!(res == 0);
        AllocPoint(ap)
    }
}

impl Drop for AllocPoint {
    fn drop(&mut self) {
        let AllocPoint(ref ap) = *self;
        unsafe {
            mps_ap_destroy(*ap);
        }
    }
}

#[inline]
fn invert_non_negative(val: u64) -> u64 {
    let mask: u64 = (!val as i64 >> 63) as u64 & !(1 << 63);
    val ^ mask
}

const TAG_POINTER_HI: u16 = 0xFFFF;
const TAG_DOUBLE_MAX: u16 = 0xFFF8;
const TAG_DOUBLE_MIN: u16 = 0x0007;
const TAG_POINTER_LO: u16 = 0x0000;

#[repr(packed, C)]
pub struct NanBox {
    val: u64,
}

impl NanBox {
    #[inline]
    pub fn tag(&self) -> u16 {
        (self.val >> 48 & 0xFFFF) as u16
    }

    #[inline]
    pub fn is_ptr(&self) -> bool {
        !self.is_nil() && (self.tag() == TAG_POINTER_LO || self.tag() == TAG_POINTER_HI)
    }

    #[inline]
    pub fn is_nil(&self) -> bool {
        self.val == 0
    }

    #[inline]
    pub fn is_double(&self) -> bool {
        self.tag() >= TAG_DOUBLE_MIN && self.tag() <= TAG_DOUBLE_MAX
    }

    #[inline]
    pub fn store_nil(&mut self) {
        self.val = 0;
        assert!(self.is_nil())
    }

    #[inline]
    pub fn store_double(&mut self, double: f64) {
        let bits: u64 = unsafe { mem::transmute(double) };
        self.val = invert_non_negative(bits);
        debug_assert!(self.is_double());
    }

    #[inline]
    pub fn load_double(&self) -> f64 {
        assert!(self.is_double());
        let bits = invert_non_negative(self.val);
        unsafe { mem::transmute(bits) }
    }

    pub fn copy_from(&mut self, other: &NanBox) {
        use std::intrinsics::volatile_copy_memory;
        use std::intrinsics::volatile_load as load;

        unsafe {
            loop {
                // FIXME: might need memory barrier
                volatile_copy_memory(self, other, 1);
                if load(&self.val) == load(&other.val) { break }
            }
        }
    }

    pub unsafe fn header(&self) -> &Header {
        debug_assert!(self.is_ptr());
        unsafe { mem::transmute(self.val) }
    }
}

pub trait ObjType {
    fn count(&self) -> usize;
    fn id(&self) -> u16;
}

pub fn alloc(dst: &mut NanBox, ty: &ObjType) {
    MPS_AMC_AP.with(|mps_amc_ap| unsafe {
        let &AllocPoint(ap) = mps_amc_ap;
        let size = (MPS_HEADER_SIZE + (ty.count() * MPS_WORD_SIZE)) as u32;
        let root: &mut mps_addr_t = mem::transmute(dst);
        let res = rust_mps_alloc_obj(root, ap, size, ty.id(), OBJ_MPS_TYPE_OBJECT);
        assert!(res == 0);
    });
}

pub struct RootTable {
    root: mps_root_t,
    base: *mut NanBox,
    count: usize,
}

impl RootTable {
    pub fn new(count: usize) -> Self {
        unsafe {
            // allocate and zero memory for table
            let size = count * mem::size_of::<NanBox>();
            let align = mem::align_of::<NanBox>();
            let base = allocate(size, align) as *mut NanBox;
            ptr::write_bytes(base, 0, count);

            // register as root
            let mut root: mps_root_t = ptr::null_mut();
            let res = rust_mps_root_create_table(&mut root, arena(),
                        base as *mut mps_addr_t,
                        count as libc::size_t);
            assert!(res == 0);

            RootTable { root: root, base: base, count: count }
        }
    }
}

impl Deref for RootTable {
    type Target = [NanBox];

    fn deref(&self) -> &[NanBox] {
        unsafe { slice::from_raw_parts(self.base, self.count) }
    }
}

impl DerefMut for RootTable {
    fn deref_mut(&mut self) -> &mut [NanBox] {
        unsafe { slice::from_raw_parts_mut(self.base, self.count) }
    }
}

impl Drop for RootTable {
    fn drop(&mut self) {
        unsafe {
            let size = self.count * mem::size_of::<NanBox>();
            let align = mem::align_of::<NanBox>();
            mps_root_destroy(self.root);
            deallocate(self.base as *mut u8, size, align);
        }
    }
}
