

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
    repr: u64
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

    // unsafe new for null pointer Nanbox
    unsafe fn new() -> NanBox {
       NanBox { repr: 0 }
    }
}

struct Arena {
    arena: mps_arena_t,
    thread: mps_thr_t,
    slots: Slots,
    slots_root: mps_root_t
}

impl Arena {
    fn new(size: uint) -> Arena {
        unsafe {
            let mut arena: mps_arena_t = mem::zeroed();
            let mut thread: mps_thr_t = mem::zeroed();
            let arenasize = size as libc::size_t;
            let res = rust_mps_create_vm_area(&mut arena, &mut thread, arenasize);
            assert!(res == 0);

            let mut slots = Slots {
                slot : mem::transmute([0u64,..VM_MAX_SLOTS]),
            };

            let mut root: mps_root_t = mem::zeroed();
            let mut base = &mut slots as *mut _ as *mut libc::c_void;
            let res = rust_mps_root_create_table(&mut root, arena, &mut base,
                                                    VM_MAX_SLOTS as libc::size_t );
            assert!(res == 0);

            Arena { arena: arena, thread: thread, slots: slots, slots_root: root}
        }
    }
}

struct ObjPool {
    ap: mps_ap_t,
    pool: mps_pool_t,
}

impl ObjPool {
    fn new(arena: Arena) -> ObjPool {
        unsafe {
            let arena_ptr: mps_arena_t = arena.arena;
            let mut pool: mps_pool_t = mem::zeroed();
            let mut ap: mps_ap_t = mem::zeroed();
            let res = rust_mps_create_obj_pool(&mut pool, &mut ap, arena_ptr);
            assert!(res == 0);

            ObjPool { ap: ap, pool: pool }
        }
    }

    fn alloc (&self, nanbox: &mut NanBox, cljtype: u16 , mpstype: u8, objsize: u32) {
        unsafe {
            let ap = self.ap;

            let res = rust_mps_alloc_obj(mem::transmute(nanbox.repr),
                                         ap,
                                         objsize,
                                         cljtype,
                                         mpstype);
            assert!(res == 0);
        }
    }
}


const VM_MAX_SLOTS : uint = 20000u;

pub struct Slots {
    pub slot : [NanBox,..VM_MAX_SLOTS],
}


#[test]
fn test_nanbox() {
    let f = 0.1234f64;

    let mut a = Arena::new(32 * 1024 * 1024);

    a.slots.slot[0].set_double(f);

    unsafe {
      rust_mps_root_destroy(a.slots_root);
    }

    assert!(a.slots.slot[0].get_double() == f);
}

