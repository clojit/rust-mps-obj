extern crate libc;

use std::intrinsics;
use std::mem;

enum mps_arena_s {}
pub type mps_arena_t = *mut mps_arena_s;

enum mps_thr_s { }
pub type mps_thr_t = *mut mps_thr_s;

enum mps_pool_s {}
pub type mps_pool_t = *mut mps_pool_s;

enum mps_ap_s {}
pub type mps_ap_t = *mut mps_ap_s;


pub type mps_addr_t = *mut libc::c_void;

pub type mps_res_t = libc::c_int;

extern {
    pub static OBJ_FMT_TYPE_PAYLOAD: u16;
    pub static OBJ_FMT_TYPE_FORWARD: u16;
    pub static OBJ_FMT_TYPE_PADDING: u16;

    pub fn rust_mps_create_vm_area(arena_o: *mut mps_arena_t,
                                    thr_o: *mut mps_thr_t,
                                    arenasize: libc::size_t) -> mps_res_t;

    pub fn rust_mps_create_obj_pool(pool_o: *mut mps_pool_t,
                                    ap_o: *mut mps_ap_t,
                                    arena: mps_arena_t) -> mps_res_t;

    pub fn rust_mps_alloc_obj(addr_o: *mut mps_addr_t,
                                ap: mps_ap_t,
                                obj: *mut libc::c_void) -> mps_res_t;

}

pub trait Info : Copy+'static {}

#[repr(C, packed)]
struct ObjStub<T: Info> {
    fmt_type: u16,
    offset: u16,
    size: u32,
    info_type: u64,
}

pub struct Obj<T: Info> {
    addr: *mut ObjStub<T>
}

#[repr(C)]
pub struct TagObj(u64);

impl TagObj {
    fn has_tag(&self) -> bool {
        true
    }

    fn as_obj<T>(&self) -> Option<Obj<T>> {
        None
    }
}

impl<T:Info> Obj<T> {

}

struct Arena {
    arena: mps_arena_t,
    thread: mps_thr_t,
}

impl Arena {
    fn new(size: uint) -> Arena {
        unsafe {
            let mut arena: mps_arena_t = mem::zeroed();
            let mut thread: mps_thr_t = mem::zeroed();
            let arenasize = size as libc::size_t;
            let res = rust_mps_create_vm_area(&mut arena, &mut thread, arenasize);
            assert!(res == 0);

            Arena { arena: arena, thread: thread }
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

    fn alloc<T:Info>(&self, info: T, len: uint) -> Obj<T> {
        unsafe {
            let ap = self.ap;

            let info_type_id = intrinsics::TypeId::of::<T>();

            // rounded up to next multiple of 8
            let info_size = (mem::size_of::<T>() + 7) & !0x7;

            // header + info + payload
            let size = 8 + info_size  + (len * 8);

            let mut obj_stub: ObjStub<T> = ObjStub {
                fmt_type: OBJ_FMT_TYPE_PAYLOAD,
                offset: info_size as u16,
                size: size as u32,
                info_type: mem::transmute(info_type_id),
            };


            let obj_ptr = &mut obj_stub as *mut _ as *mut libc::c_void;

            // TODO: need to root addr before allocating it!!
            let mut addr: mps_addr_t = mem::zeroed();
            let res = rust_mps_alloc_obj(&mut addr, ap, obj_ptr);
            assert!(res == 0);

            Obj { addr: addr as *mut ObjStub<T> }
        }
    }
}



#[test]
fn create_arena() {
    let a = Arena::new(32 * 1024 * 1024);

    let p = ObjPool::new(a);
}
