#![allow(non_camel_case_types)]
#![allow(dead_code)] // FIXME

extern crate libc;

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

enum mps_fmt_s {}
pub type mps_fmt_t = *mut mps_fmt_s;

pub type mps_addr_t = *mut libc::c_void;

pub type mps_res_t = libc::c_int;

extern {
    pub static OBJ_MPS_TYPE_PADDING: u8;
    pub static OBJ_MPS_TYPE_FORWARD: u8;
    pub static OBJ_MPS_TYPE_OBJECT : u8;
    pub static OBJ_MPS_TYPE_ARRAY  : u8;

    pub fn rust_mps_create_vm_area(arena_o: *mut mps_arena_t,
                                    arenasize: libc::size_t) -> mps_res_t;

    pub fn rust_mps_create_amc_pool(pool_o: *mut mps_pool_t,
                                    fmt_o: *mut mps_fmt_t,
                                    arena: mps_arena_t) -> mps_res_t;

    pub fn rust_mps_alloc_obj(addr_o: *mut mps_addr_t,
                                ap: mps_ap_t,
                                size: u32, cljtype: u16, mpstype: u8) -> mps_res_t;

    pub fn rust_mps_root_create_table(root_o: *mut mps_root_t,
                                      arena: mps_arena_t,
                                      base: *mut mps_addr_t,
                                      count: libc::size_t) -> mps_res_t;

    pub fn rust_mps_create_ap(ap_o: *mut mps_ap_t, pool: mps_pool_t) -> mps_res_t;

    pub fn rust_mps_debug_print_reachable(arena: mps_arena_t, fmt: mps_fmt_t);

    pub fn mps_thread_reg(thr_o: *mut mps_thr_t, arena: mps_arena_t) -> mps_res_t;

    pub fn mps_root_destroy(root: mps_root_t);
    pub fn mps_ap_destroy(ap: mps_ap_t);
    pub fn mps_thread_dereg(thr: mps_thr_t);
    pub fn mps_pool_destroy(pool: mps_pool_t);
    pub fn mps_fmt_destroy(fmt: mps_fmt_t);
    pub fn mps_arena_destroy(arena: mps_arena_t);

    pub fn mps_arena_collect(arena: mps_arena_t);
    pub fn mps_arena_release(arena: mps_arena_t);
}
