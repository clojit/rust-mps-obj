#![feature(libc, std_misc, alloc, core)]

use std::fmt;

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
    let mut slot = mps::RootTable::new(4096);
    mps::debug_printwalk();

    let ty = CljType { id: 42, count: 3 };
    mps::alloc(&mut slot[0], &ty);
    mps::debug_printwalk();
    let val = slot[0].load(); // copy object to rust stack
    slot[0].store(mps::Value::Nil);
    mps::debug_printwalk();
    mps::gc();
    mps::debug_printwalk();
}
