#![feature(libc, std_misc, alloc, core, unsafe_destructor)]

use std::fmt;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::collections::{BitVec};
use std::cell::RefCell;
use std::slice;

use mps::NanBox;

mod mps;

// dynamic nanbox
// objref (pointer, rooted, does not borrow nanbox)
// Value = int|double|objref
// nanbox.load() -> Value
// nanbox.store(Value)
// objref.getfield(idx)
//      1. *(ptr + idx)
//      2. do check if ptr is still valid



pub struct CljType {
    id: u16,
    count: u32,
}

impl mps::ObjType for CljType {
    fn count(&self) -> usize { self.count as usize }
    fn id(&self) -> u16 { self.id }
}

const SCRATCH_TABLE_SIZE: usize = 128;

struct ScratchTable {
    table: mps::RootTable,
    free: BitVec,
}

impl ScratchTable {
    fn alloc(&mut self) -> usize {
        let index = self.free.iter()
                        .position(|isfree| isfree)
                        .expect("Out of scratch registers!");
        self.free.set(index, false);

        index
    }

    fn free(&mut self, index: usize) {
        self.free.set(index, false);
    }
}

thread_local!{
    static SCRATCH: RefCell<ScratchTable> = RefCell::new(ScratchTable {
        table: mps::RootTable::new(SCRATCH_TABLE_SIZE),
        free: BitVec::from_elem(SCRATCH_TABLE_SIZE, true),
    })
}

#[repr(packed, C)]
pub struct ObjRef {
    ptr: *mut mps::NanBox,
}

// ObjRef is pointer to NanBox which is guaranteed to contain a rooted pointer
impl ObjRef {
    pub fn new(from: &NanBox) -> Self {
        SCRATCH.with(|cell| {
            let mut scratch = cell.borrow_mut();
            let index = scratch.alloc();

            scratch.table[index].copy_from(from);

            ObjRef { ptr: &mut scratch.table[index] }
        })
    }

    fn header(&self) -> &mps::Header {
        let nanbox: &NanBox = self;
        unsafe { nanbox.header() }
    }
}

impl Drop for ObjRef {
    fn drop(&mut self) {
        SCRATCH.with(|cell| {
            let mut scratch = cell.borrow_mut();
            let base: *mut NanBox = scratch.table.as_mut_ptr();
            scratch.free(self.ptr as usize - base as usize);
        })
    }
}

impl Deref for ObjRef {
    type Target = mps::NanBox;

    fn deref(&self) -> &mps::NanBox {
        unsafe { self.ptr.as_ref().unwrap() }
    }
}

impl DerefMut for ObjRef {
    fn deref_mut(&mut self) -> &mut mps::NanBox {
        unsafe { self.ptr.as_mut().unwrap() }
    }
}

pub enum Value {
    Nil,
    //Int(i32),
    Float(f64),
    Obj(ObjRef),
}

trait Slot {
    fn store(&mut self, value: &Value);
    fn load(&self) -> Value;
}

impl Slot for mps::NanBox {
    fn store(&mut self, value: &Value) {
        match *value {
            Value::Nil => {
                self.store_nil();
            },
            Value::Float(double) => {
                self.store_double(double);
            },
            Value::Obj(ref other) => {
                self.copy_from(other);
            }
        }
    }

    fn load(&self) -> Value {
        if self.is_nil() {
            Value::Nil
        } else if self.is_double() {
            Value::Float(self.load_double())
        } else if self.is_ptr() {
            Value::Obj(ObjRef::new(self))
        } else {
            unreachable!()
        }
    }
}

/*

impl<'a> ObjRef<'a> {
    unsafe fn field(&self, idx: u16) -> *mut NanBox {
        let obj: *mut NanBox = mem::transmute(self.borrow.repr);
        obj.offset(1 + (idx as isize))
        // TODO size check
    }

    pub fn getfield(&self, idx: u16) -> RawValue<'a> {
        unsafe {
            let field = self.field(idx);
            RawValue { borrow: field.as_ref().unwrap() }
        }
    }

    pub fn setfield(&mut self, idx: u16, value: Value) {
        unsafe {
            let field = self.field(idx);
            (&mut *field).store(value);
        }
    }
}
*/

fn main() {
    let mut slot: mps::RootTable = mps::RootTable::new(4096);
    mps::debug_printwalk();

    let ty = CljType { id: 42, count: 3 };
    mps::alloc(&mut slot[0], &ty);
    mps::debug_printwalk();
    let val = slot[0].load(); // copy object to rust stack
    if let Value::Obj(obj) = val {
        println!("val obj len: {}", obj.header().len());
    }
    slot[0].store(&Value::Nil);
    mps::debug_printwalk();
    mps::gc();
    mps::debug_printwalk();
}
