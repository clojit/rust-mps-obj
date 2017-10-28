use std::sync::Arc;
use std::os::raw;
use std::ptr;

use ffi::*;
use errors::{Error, Result};
use arena::Arena;

pub struct Format {
    _arena: Arena,
    ptr: Arc<RawFormat>,
}

impl Format {
    pub unsafe fn from_raw(arena: Arena, fmt: mps_fmt_t) -> Self {
        Format {
            _arena: arena,
            ptr: Arc::new(RawFormat { fmt }),
        }
    }

    pub unsafe fn as_raw_ptr(&self) -> mps_fmt_t {
        self.ptr.fmt
    }
}

#[repr(u8)]
enum Content {
    Padding = 0,
    Forward = 1,
    Object = 2,
}

#[repr(C, packed)]
struct Header {
    content: Content,
    _reserved: u8,
    class: u16,
    length: u32,
}

/// Vector of words
pub struct AreaFormat {
    fmt: Format,
}

pub trait ReferenceFormat {
    const MASK: u64;
    const PATTERN: u64;
}

unsafe extern "C" fn obj_scan_tagged<R: ReferenceFormat>(
    ss: mps_ss_t,
    base: mps_addr_t,
    limit: mps_addr_t,
) -> mps_res_t {
    // This is the place where the magic happens, this relies on associated generics and
    // rvalue static promtion to essentially create a stateless scan function for
    // each used reference format.
    let scan_tag: &'static mps_scan_tag_s = &mps_scan_tag_s {
        mask: R::MASK,
        pattern: R::PATTERN,
    };

    let mut base = base;
    let closure = scan_tag as *const _ as *mut raw::c_void;

    while base < limit {
        let obj = base as *mut Header;
        let obj_base = obj.offset(1) as mps_addr_t;
        let obj_limit = obj_skip(base);

        if let Content::Object = (*obj).content {
            let res = mps_scan_area_tagged(ss, obj_base, obj_limit, closure);
            if res != MPS_RES_OK as mps_res_t {
                return res;
            }
        }

        base = obj_limit;
    }


    MPS_RES_OK as mps_res_t
}

unsafe extern "C" fn obj_skip(base: mps_addr_t) -> mps_addr_t {
    let obj = base as *mut Header;
    return base.offset((*obj).length as isize);
}

unsafe extern "C" fn obj_isfwd(base: mps_addr_t) -> mps_addr_t {
    let obj = base as *mut Header;
    if let Content::Forward = (*obj).content {
        let fwd = obj.offset(1) as *mut mps_addr_t;
        return *fwd;
    }

    return ptr::null_mut();
}

unsafe extern "C" fn obj_fwd(base: mps_addr_t, new: mps_addr_t) {
    let obj = base as *mut Header;
    let fwd = obj.offset(1) as *mut mps_addr_t;
    (*obj).content = Content::Forward;
    *fwd = new;
}

unsafe extern "C" fn obj_pad(base: mps_addr_t, length: usize) {
    let obj = base as *mut Header;
    (*obj).content = Content::Padding;
    (*obj).length = length as u32
}

impl AreaFormat {
    pub fn tagged<R: ReferenceFormat, A: AsRef<Arena>>(arena: A) -> Result<Self> {
        let arena = arena.as_ref().clone();
        let args = mps_args! {
            MPS_KEY_FMT_SCAN: Some(obj_scan_tagged::<R>),
            MPS_KEY_FMT_SKIP: Some(obj_skip),
            MPS_KEY_FMT_FWD: Some(obj_fwd),
            MPS_KEY_FMT_ISFWD: Some(obj_isfwd),
            MPS_KEY_FMT_PAD: Some(obj_pad),
        };

        let format = unsafe {
            let mut format: mps_fmt_t = ptr::null_mut();
            let res = mps_fmt_create_k(&mut format, arena.as_raw_ptr(), args);
            Error::result(res).map(|_| Format::from_raw(arena, format))
        }?;

        Ok(AreaFormat { fmt: format })
    }
}

impl AsRef<Format> for AreaFormat {
    fn as_ref(&self) -> &Format {
        &self.fmt
    }
}

struct RawFormat {
    fmt: mps_fmt_t,
}

impl Drop for RawFormat {
    fn drop(&mut self) {
        unsafe {
            mps_fmt_destroy(self.fmt);
        }
    }
}
