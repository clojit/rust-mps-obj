#![feature(globs)]
#![feature(unsafe_destructor)]

extern crate libc;
use std::kinds::marker;
use std::mem;
use std::os::{MemoryMap,MapOption};
use std::raw::Slice as RawSlice;

use ffi::*;

mod ffi;

/*
#[repr(packed, C)]
struct ObjStub {
    mpstype: u8,
    unused: u8,
    cljtype: u16,
    size: u32
}*/

#[repr(packed, C)]
#[deriving(PartialEq, Show)]
#[allow(missing_copy_implementations)]
pub struct NanBox {
    repr: u64,
}

#[deriving(PartialEq, Show)]
// reference to NanBox which is guaranteed to contain a pointer to an object
pub struct ObjRef<'a>(&'a NanBox);

#[deriving(PartialEq, Show)]
pub enum Value<'a> {
    Nil,
    //Int(i32),
    Float(f64),
    Obj(ObjRef<'a>)
}

impl<'a> Value<'a> {
    pub fn float(self) -> f64 {
        match self {
            Value::Float(val) => val,
            _ => panic!("Expected Float, got {:?}", self)
        }
    }

    pub fn obj(self) -> ObjRef<'a> {
        match self {
            Value::Obj(val) => val,
            _ => panic!("Expected Obj, got {:?}", self)
        }
    }

    pub fn nil(self) -> () {
        match self {
            Value::Nil => (),
            _ => panic!("Expected Obj, got {:?}", self)
        }
    }
}

/*pub struct CljType {
    name: String,
    id: u16,
    size: u32,
}*/

#[inline]
fn invert_non_negative(repr: u64) -> u64 {
    let mask: u64 = (!repr as i64 >> 63) as u64 & !(1 << 63);
    repr ^ mask
}

const TAG_POINTER_HI: u16 = 0xFFFF;
const TAG_DOUBLE_MAX: u16 = 0xFFF8;
const TAG_DOUBLE_MIN: u16 = 0x0007;
const TAG_POINTER_LO: u16 = 0x0000;

impl NanBox {
    #[inline]
    fn tag(&self) -> u16 {
        (self.repr >> 48 & 0xFFFF) as u16
    }

    #[inline]
    fn is_objref(&self) -> bool {
        !self.is_nil() && (self.tag() == TAG_POINTER_LO || self.tag() == TAG_POINTER_HI)
    }

    #[inline]
    fn is_nil(&self) -> bool {
        self.repr == 0
    }

    #[inline]
    fn is_double(&self) -> bool {
        self.tag() >= TAG_DOUBLE_MIN && self.tag() <= TAG_DOUBLE_MAX
    }

    pub fn store(&mut self, value: Value) {
        match value {
            Value::Nil => {
                self.repr = 0;
                assert!(self.is_nil())
            },
            Value::Float(double) => {
                let bits: u64 = unsafe { mem::transmute(double) };
                self.repr = invert_non_negative(bits);
                assert!(self.is_double());
            },
            Value::Obj(ObjRef(other)) => {
                self.repr = other.repr;
                assert!(self.is_objref());
            }
        }
    }

    pub fn load(&self) -> Value {
        if self.is_nil() {
            Value::Nil
        } else if self.is_double() {
            let bits = invert_non_negative(self.repr);
            Value::Float(unsafe { mem::transmute(bits) })
        } else if self.is_objref() {
            Value::Obj(ObjRef(self))
        } else {
            unreachable!()
        }
    }

/*
    fn alloc_obj(&mut self, ap: mps_ap_t, cljtype: u16, count: u32) {
        unsafe {
            // size in bytes, including header
            let size = 8 + (count * 8);
            let res = rust_mps_alloc_obj(mem::transmute(&mut self.repr),
                                         ap,
                                         size,
                                         cljtype,
                                         OBJ_MPS_TYPE_OBJECT);
            assert!(res == 0);
        }
    }

    fn get_field(&mut self, idx: u16) -> &mut NanBox {
        unsafe {
            assert!(self.is_objref());
            let self_ptr = self as *mut NanBox;
            let field_ptr: *mut NanBox = self_ptr.offset(1 + (idx as int));

            // RawPtr.as_ref() returns immutable &NanBox, even for *mut NanBox
            mem::transmute(field_ptr)
        }
    }

    fn replace(&mut self, other: &mut NanBox) {
        self.repr = other.repr;
    }
*/
}

pub struct MemoryPoolSystem {
    arena: mps_arena_t,
    amc: mps_pool_t,
    fmt: mps_fmt_t,
    // global ns
    //ns_root: mps_root_t
}

impl MemoryPoolSystem {
    pub fn new(heapsize: uint) -> MemoryPoolSystem {
        // create arena of given size
        let arena = unsafe {
            let mut arena: mps_arena_t = mem::zeroed();
            let arenasize = heapsize as libc::size_t;
            let res = rust_mps_create_vm_area(&mut arena, arenasize);
            assert!(res == 0);

            arena
        };

        // create AMC pool and object format
        let (amc, fmt) = unsafe {
            let mut amc = mem::zeroed();
            let mut fmt = mem::zeroed();
            let res = rust_mps_create_amc_pool(&mut amc, &mut fmt, arena);
            assert!(res == 0);

            (amc, fmt)
        };

        MemoryPoolSystem { arena: arena, amc: amc, fmt: fmt }
    }
}

impl Drop for MemoryPoolSystem {
    fn drop(&mut self) {
        // at this point, all other aps and roots are already dropped
        // TODO: do we need to park the arena first?
        unsafe {
            mps_pool_destroy(self.amc);
            mps_fmt_destroy(self.fmt);
            mps_arena_destroy(self.arena);
        }
    }
}

pub type Slot = NanBox;

pub struct MemoryContext<'a> {
    thread: mps_thr_t,
    amc_ap: mps_ap_t,
    pub slot: Slots<'a>,
}

pub struct Slots<'a> {
    root: mps_root_t,
    _mmap: MemoryMap,
    slice: &'a mut [Slot],

    marker: marker::ContravariantLifetime<'a>,
}

impl<'a> MemoryContext<'a> {

    pub fn new(slots: Slots<'a>, mps: &'a MemoryPoolSystem) -> MemoryContext<'a> {
        let thread = unsafe {
            let mut thread = mem::zeroed();
            let res = mps_thread_reg(&mut thread, mps.arena);
            assert!(res == 0);
            thread
        };

        let amc_ap = unsafe {
            let mut ap = mem::zeroed();
            let res = rust_mps_create_ap(&mut ap, mps.amc);
            assert!(res == 0);
            ap
        };

        MemoryContext {
            thread: thread,
            amc_ap: amc_ap,
            slot: slots
        }
    }

}

#[unsafe_destructor]
impl<'a> Drop for MemoryContext<'a> {
    fn drop(&mut self) {
        // FIXME: is it safe to deregister the thread before the roots?
        unsafe {
            mps_ap_destroy(self.amc_ap);
            mps_thread_dereg(self.thread);
        }
    }
}

impl<'a> Slots<'a> {
    pub fn new(slot_count: uint, mps: &'a MemoryPoolSystem) -> Slots<'a> {
        let mmap = {
            let size = slot_count * mem::size_of::<Slot>();
            let options = [MapOption::MapReadable, MapOption::MapWritable];
            MemoryMap::new(size, &options).unwrap()
        };

        let slice: &'a mut [Slot] = unsafe {
            mem::transmute(RawSlice {
                data: mmap.data() as *const Slot, len: slot_count
            })
        };

        let root = unsafe {
            let mut root = mem::zeroed();
            let res = rust_mps_root_create_table(&mut root, mps.arena,
                                      mmap.data() as *mut mps_addr_t,
                                      slot_count as libc::size_t);
            assert!(res == 0);
            root
        };

        Slots {
            root: root,
            _mmap: mmap,
            slice: slice,
            marker: marker::ContravariantLifetime,
        }
    }
}

#[unsafe_destructor]
impl<'a> Drop for Slots<'a> {
    fn drop(&mut self) {
        unsafe {
            mps_root_destroy(self.root);
        }
    }
}

impl<'a> Index<uint, Slot> for Slots<'a> {
    #[inline]
    fn index(&self, index: &uint) -> &Slot {
        &self.slice[*index]
    }
}

impl<'a> IndexMut<uint, Slot> for Slots<'a> {
    #[inline]
    fn index_mut(&mut self, index: &uint) -> &mut Slot {
        &mut self.slice[*index]
    }
}

fn main() {
    let mps = MemoryPoolSystem::new(32*1024*1024);
    let slots = Slots::new(2048, &mps);
    let mut ctx = MemoryContext::new(slots, &mps);

    ctx.slot[0].store(Value::Float(3.14));
    println!("{}", ctx.slot[0].load().float());
}
