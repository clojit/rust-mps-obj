#![feature(libc, std_misc, alloc)]

use std::fmt;
use std::mem;

use ffi::*;

mod ffi;
mod mps;

pub const HEADER_SIZE: u32 = 8;
pub const NANBOX_SIZE: u32 = 8;

// reference to NanBox which is guaranteed to contain a pointer to an object
pub struct ObjRef<'a> {
    borrow: &'a NanBox,
}

pub enum Value<'a> {
    Nil,
    //Int(i32),
    Float(f64),
    Obj(ObjRef<'a>),
}


impl<'a> fmt::Debug for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Value::Nil           => write!(f, "Nil"),
            &Value::Float(double) => write!(f, "Float({})", double),
            &Value::Obj(ref obj)  => write!(f, "Obj(0x{:x})", obj.borrow.repr),
        }
    }
}

/*
impl<'a> Value<'a> {
    pub fn float(self) -> f64 {
        match self {
            Value::Float(val) => val,
            _ => panic!("Expected Float, got {:?}", self)
        }
    }

    pub fn obj(&self) -> ObjRef<'a> {
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
}*/

pub struct CljType {
    id: u16,
    count: u32,
}

impl mps::ObjType for CljType {
    fn count(&self) -> usize { self.count as usize }
    fn id(&self) -> u16 { self.id }
}


#[inline]
fn invert_non_negative(repr: u64) -> u64 {
    let mask: u64 = (!repr as i64 >> 63) as u64 & !(1 << 63);
    repr ^ mask
}

const TAG_POINTER_HI: u16 = 0xFFFF;
const TAG_DOUBLE_MAX: u16 = 0xFFF8;
const TAG_DOUBLE_MIN: u16 = 0x0007;
const TAG_POINTER_LO: u16 = 0x0000;

#[repr(packed, C)]
pub struct NanBox {
    pub repr: u64,
}

impl NanBox {
    #[inline]
    pub fn tag(&self) -> u16 {
        (self.repr >> 48 & 0xFFFF) as u16
    }

    #[inline]
    pub fn is_ptr(&self) -> bool {
        !self.is_nil() && (self.tag() == TAG_POINTER_LO || self.tag() == TAG_POINTER_HI)
    }

    #[inline]
    pub fn is_nil(&self) -> bool {
        self.repr == 0
    }

    #[inline]
    pub fn is_double(&self) -> bool {
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
            Value::Obj(other) => {
                self.repr = other.borrow.repr;
                assert!(self.is_ptr());
            }
        }
    }

    pub fn load(&self) -> Value {
        if self.is_nil() {
            Value::Nil
        } else if self.is_double() {
            let bits = invert_non_negative(self.repr);
            Value::Float(unsafe { mem::transmute(bits) })
        } else if self.is_ptr() {
            Value::Obj(ObjRef { borrow: self })
        } else {
            unreachable!()
        }
    }
}

impl<'a> ObjRef<'a> {
    unsafe fn field(&self, idx: u16) -> *mut NanBox {
        let obj: *mut NanBox = mem::transmute(self.borrow.repr);
        obj.offset(1 + (idx as isize))
        // TODO size check
    }

    /*pub fn getfield(&self, idx: u16) -> RawValue<'a> {
        unsafe {
            let field = self.field(idx);
            RawValue { borrow: field.as_ref().unwrap() }
        }
    }*/

    pub fn setfield(&mut self, idx: u16, value: Value) {
        unsafe {
            let field = self.field(idx);
            (&mut *field).store(value);
        }
    }
}

/*
pub type Slot = NanBox;

pub struct Context<'a> {
    pub slot: Slots<'a>,

    // allocation point for objects
    amc_ap: mps_ap_t,

    // root memory for slot and scratch
    #[allow(dead_code)]
    mmap: MemoryMap,

    // registered root for slots array
    root: mps_root_t,

    // mps thread which is represented by this context
    thread: mps_thr_t,
}

pub struct Slots {
    // base offset for slots array
    base: usize,
    slots: &'a mut [Slot],
}

impl<'a> Context<'a> {

    pub fn new(slot_count: usize, mps: &'a MemoryPoolSystem) -> Context<'a> {
        let thread = unsafe {
            let mut thread = mem::zeroed();
            let res = mps_thread_reg(&mut thread, mps.arena);
            assert!(res == 0);
            thread
        };

        let mmap = {
            let size = slot_count * mem::size_of::<Slot>();
            let options = [MapOption::MapReadable, MapOption::MapWritable];
            MemoryMap::new(size, &options).unwrap()
        };

        let root = unsafe {
            let mut root = mem::zeroed();
            let res = rust_mps_root_create_table(&mut root, mps.arena,
                                      mmap.data() as *mut mps_addr_t,
                                      slot_count as libc::size_t);
            assert!(res == 0);
            root
        };

        let slice : &'a mut [Slot] = unsafe {
            mem::transmute(RawSlice {
                data: mmap.data() as *const Slot,
                len: slot_count
            })
        };

        let amc_ap = unsafe {
            let mut ap = mem::zeroed();
            let res = rust_mps_create_ap(&mut ap, mps.amc);
            assert!(res == 0);
            ap
        };

        Context {
            thread: thread,
            mmap: mmap,
            root: root,
            amc_ap: amc_ap,

            slot: Slots {
                base: 0,
                slots: slice,
            },
        }
    }

}

impl<'a> Drop for Context<'a> {
    fn drop(&mut self) {
        unsafe {
            mps_ap_destroy(self.amc_ap);
            mps_root_destroy(self.root);
            mps_thread_dereg(self.thread);
        }
    }
}

impl<'a> Slots<'a> {
    // FIXME: with the IndexSet trait, we can make this nicer
    pub fn alloc(&mut self, dst: usize, cljtype: &CljType) {
        unsafe {
            let ap = self.amc_ap;
            let dst = &mut self[dst];
            // size in bytes, including header
            let size = HEADER_SIZE + (cljtype.size * NANBOX_SIZE);
            let res = rust_mps_alloc_obj(mem::transmute(&mut dst.repr),
                                         ap,
                                         size,
                                         cljtype.id,
                                         OBJ_MPS_TYPE_OBJECT);
            assert!(res == 0);
        }
    }
}

impl<'a> Index<usize> for Slots<'a> {
    type Output = Slot;
    #[inline]
    fn index(&self, index: &usize) -> &Slot {
        &self.slots[self.base + *index]
    }
}

impl<'a> IndexMut<usize> for Slots<'a> {
    #[inline]
    fn index_mut(&mut self, index: &usize) -> &mut Slot {
        &mut self.slots[self.base + *index]
    }
}
*/


fn main() {
    let mut rt = mps::RootTable::new(128);
    mps::debug_printwalk();

    let ty = CljType { id: 42, count: 3 };
    mps::alloc(&mut rt[0], &ty);
    mps::debug_printwalk();
    rt[0] = unsafe { mem::zeroed() };
    mps::debug_printwalk();
    mps::gc();
    mps::debug_printwalk();
}
