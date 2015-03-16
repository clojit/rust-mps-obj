extern crate libc;

use std::rt::at_exit;
use std::mem;
use std::ptr;
use std::slice;
use std::ops::{Deref, DerefMut};
use std::sync::{Once, ONCE_INIT};
use std::rt::heap::{allocate, deallocate};

use ffi::*;

const MPS_HEAP_SIZE: usize = 32*1024*1024;

const MPS_HEADER_SIZE: usize = 8;
const MPS_WORD_SIZE: usize = 8;

fn arena() -> mps_arena_t {
    static mut arena: mps_arena_t = 0 as mps_arena_t;
    static INIT: Once = ONCE_INIT;
    INIT.call_once(|| unsafe {
        let arenasize = MPS_HEAP_SIZE as libc::size_t;
        let res = rust_mps_create_vm_area(&mut arena, arenasize);
        assert!(res == 0);
    });

    at_exit(|| unsafe {
        mps_arena_destroy(arena);
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
    });

    at_exit(|| unsafe {
        mps_pool_destroy(amc);
        mps_fmt_destroy(fmt);
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

pub trait ObjType {
    fn count(&self) -> usize;
    fn id(&self) -> u16;
}

#[repr(packed, C)]
pub struct RootedPtr(mps_addr_t);

pub fn alloc(dst: &mut RootedPtr, ty: &ObjType) {
    MPS_AMC_AP.with(|mps_amc_ap| unsafe {
        let &AllocPoint(ap) = mps_amc_ap;
        let RootedPtr(ref mut root) = *dst;
        let size = (MPS_HEADER_SIZE + (ty.count() * MPS_WORD_SIZE)) as u32;
        let res = rust_mps_alloc_obj(root, ap, size, ty.id(), OBJ_MPS_TYPE_OBJECT);
        assert!(res == 0);
    });
}

pub struct RootTable {
    root: mps_root_t,
    base: *mut RootedPtr,
    count: usize,
}

impl RootTable {
    pub fn new(count: usize) -> RootTable {
        unsafe {
            // allocate and zero memory for table
            let size = count * mem::size_of::<RootedPtr>();
            let align = mem::align_of::<RootedPtr>();
            let base = allocate(size, align) as *mut RootedPtr;
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
    type Target = [RootedPtr];

    fn deref(&self) -> &[RootedPtr] {
        unsafe { slice::from_raw_parts(self.base, self.count) }
    }
}

impl DerefMut for RootTable {
    fn deref_mut(&mut self) -> &mut [RootedPtr] {
        unsafe { slice::from_raw_parts_mut(self.base, self.count) }
    }
}

impl Drop for RootTable {
    fn drop(&mut self) {
        unsafe {
            let size = self.count * mem::size_of::<RootedPtr>();
            let align = mem::align_of::<RootedPtr>();
            mps_root_destroy(self.root);
            deallocate(self.base as *mut u8, size, align);
        }
    }
}
