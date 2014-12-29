

extern crate libc;

use std::mem;

enum mps_arena_s {}
pub type mps_arena_t = *mut mps_arena_s;

enum mps_thr_s { }
pub type mps_thr_t = *mut mps_thr_s;

enum mps_pool_s {}
pub type mps_pool_t = *mut mps_pool_s;


enum mps_root_s { }
pub type mps_root_t = *mut mps_root_s;

enum mps_ap_s {}
pub type mps_ap_t = *mut mps_ap_s;


pub type mps_addr_t = *mut libc::c_void;

pub type mps_res_t = libc::c_int;

extern {
    pub static OBJ_MPS_TYPE_PADDING: u8;
    pub static OBJ_MPS_TYPE_FORWARD: u8;
    pub static OBJ_MPS_TYPE_OBJECT : u8;
    pub static OBJ_MPS_TYPE_ARRAY  : u8;

    pub fn rust_mps_create_vm_area(arena_o: *mut mps_arena_t,
                                    thr_o: *mut mps_thr_t,
                                    arenasize: libc::size_t) -> mps_res_t;

    pub fn rust_mps_create_obj_pool(pool_o: *mut mps_pool_t,
                                    ap_o: *mut mps_ap_t,
                                    arena: mps_arena_t) -> mps_res_t;

    pub fn rust_mps_alloc_obj(addr_o: *mut mps_addr_t,
                                ap: mps_ap_t,
                                size: u32, cljtype: u16, mpstype: u8) -> mps_res_t;

    pub fn rust_mps_root_create_table(root_o: *mut mps_root_t,
                                      arena: mps_arena_t,
                                      base: *mut mps_addr_t,
                                      count: libc::size_t) -> mps_res_t;

    pub fn rust_mps_root_destroy(root_o: mps_root_t);

}

#[repr(packed, C)]
struct ObjStub {
    mpstype: u8,
    unused: u8,
    cljtype: u16,
    size: u32
}

#[repr(packed, C)]
pub struct NanBox {
    repr: u64,
    _marker: std::kinds::marker::NoCopy
}

pub struct CljType {
    name: String,
    id: u16,
    size: u32,
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

impl NanBox {
    #[inline]
    fn tag(&self) -> u16 {
        (self.repr >> 48 & 0xFFFF) as u16
    }

    // ObjRef
    #[inline]
    fn is_objref(&self) -> bool {
        self.tag() == TAG_POINTER_LO || self.tag() == TAG_POINTER_HI
    }

    fn is_null(&self) -> bool {
        self.repr == 0
    }

    // Double
    #[inline]
    fn is_double(&self) -> bool {
        self.tag() >= TAG_DOUBLE_MIN && self.tag() <= TAG_DOUBLE_MAX
    }

    #[inline]
    fn get_double(&self) -> f64 {
        assert!(self.is_double());
        let bits = invert_non_negative(self.repr);
        unsafe { mem::transmute(bits) }
    }

    #[inline]
    fn set_double(&mut self, double: f64) {
        let bits: u64 = unsafe { mem::transmute(double) };
        self.repr = invert_non_negative(bits);

        assert!(self.is_double());
    }

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
}

struct Arena {
    arena : mps_arena_t,
    thread: mps_thr_t,
    slots : Slots,
    pools : Pools
}

struct Pools {
    amc: ObjPool
}

struct ObjPool {
    ap: mps_ap_t,
    pool: mps_pool_t,
}

pub struct Slots {
    pub slot : [NanBox,..VM_MAX_SLOTS],
    root: mps_root_t
}


/*
TODO
impl Index for Slots {
    ...
}
*/


impl Arena {
    fn new(size: uint) -> Arena {
        unsafe {
            let mut arena: mps_arena_t = mem::zeroed();
            let mut thread: mps_thr_t = mem::zeroed();
            let arenasize = size as libc::size_t;
            let res = rust_mps_create_vm_area(&mut arena, &mut thread, arenasize);
            assert!(res == 0);

            let mut slots = Slots {
                slot: mem::transmute([0u64,..VM_MAX_SLOTS]),
                root: mem::zeroed()
            };

            let base: *mut mps_addr_t = mem::transmute(&mut slots.slot);
            let res = rust_mps_root_create_table(&mut slots.root, arena,
                                                 base,
                                                 VM_MAX_SLOTS as libc::size_t );
            assert!(res == 0);

            let pools = Pools {
                amc: ObjPool::new(arena)
            };

            Arena { arena: arena, thread: thread, slots: slots, pools:pools}
        }
    }
}



impl ObjPool {
    fn new(arena : mps_arena_t) -> ObjPool {
        unsafe {
            let mut pool: mps_pool_t = mem::zeroed();
            let mut ap: mps_ap_t = mem::zeroed();
            let res = rust_mps_create_obj_pool(&mut pool, &mut ap, arena);
            assert!(res == 0);

            ObjPool { ap: ap, pool: pool }
        }
    }


}

const VM_MAX_SLOTS : uint = 20000u;


/*
#[test]
fn test_nanbox() {
    let f = 0.1234f64;

    let mut a = Arena::new(32 * 1024 * 1024);

    a.slots.slot[0].set_double(f);

    unsafe {
      rust_mps_root_destroy(a.slots_root);
    }

    assert!(a.slots.slot[0].get_double() == f);
}*/


#[test]
fn test_nanbox() {
    let f = 0.1234f64;

    let mut arena = Arena::new(32 * 1024 * 1024);

    // allocate object of type 1 with 3 fields
    let nanbox: &mut NanBox = &mut arena.slots.slot[0];
    nanbox.alloc_obj(arena.pools.amc.ap, 1, 3);
    {
        let field = nanbox.get_field(0);
        assert!(field.is_null());

        field.set_double(2.342);
        assert!(field.is_double());
        assert!(field.get_double() == 2.342);

        //let tmp: &mut NanBox = &mut arena.slots.slot[1];
    }
    assert!(nanbox.is_objref());

    unsafe {
        rust_mps_root_destroy(arena.slots.root);
    }
}



