//! Custom object format for vector of words.

use std::sync::Arc;
use std::os::raw;
use std::ptr;

use errors::{Error, Result};
use fmt::{Format, RawFormat};
use arena::{Arena};

use ffi::*;

/// A custom object format for dynamically sized object areas.
///
/// An area is a vector of words, all of which could be references. The size of
/// the area can be different for every object, but it has to be specified
/// when an object is allocated and cannot be chanced afterwards.
/// This object format supports tagged references, which have to be described
/// statically using the `ReferenceTag` trait.
#[derive(Clone)]
pub struct AreaFormat<A> {
    raw: Arc<RawFormat>,
    arena: A,
}

/// Describes the format of a tagged reference.
///
/// This is used to find and fix references during scanning. Refer to
/// Memory Pool System reference about
/// [area scanners](http://www.ravenbrook.com/project/mps/master/manual/html/topic/scanning.html#area-scanners)
/// for more details.
pub trait ReferenceTag {
    /// Mask to extract the tag bits from a tagged reference
    const MASK: u64;
    /// A value is only considered a reference if the tag matches this pattern.
    const PATTERN: u64;
}

impl<A: Arena> AreaFormat<A> {
    /// Creates a new object format which will be scanned using the built-in
    /// `mps_scan_area_tagged` area scanner.
    pub fn tagged<R: ReferenceTag>(arena: &A) -> Result<Self> {
        let arena = arena.clone();
        let args = mps_args! {
            MPS_KEY_FMT_SCAN: Some(obj_scan_tagged::<R>),
            MPS_KEY_FMT_SKIP: Some(obj_skip),
            MPS_KEY_FMT_FWD: Some(obj_fwd),
            MPS_KEY_FMT_ISFWD: Some(obj_isfwd),
            MPS_KEY_FMT_PAD: Some(obj_pad),
        };

        let format = unsafe {
            let mut fmt: mps_fmt_t = ptr::null_mut();
            let res = mps_fmt_create_k(&mut fmt, arena.as_raw(), args);
            Error::result(res).map(|_| RawFormat { fmt })
        }?;

        Ok(AreaFormat {
            raw: Arc::new(format),
            arena: arena.clone(),
        })
    }
}

impl<A: Arena> Format for AreaFormat<A> {
    type Arena = A;

    fn as_raw(&self) -> mps_fmt_t {
        self.raw.fmt
    }
    
    fn arena(&self) -> &Self::Arena {
        &self.arena
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

unsafe extern "C" fn obj_scan_tagged<R: ReferenceTag>(
    ss: mps_ss_t,
    base: mps_addr_t,
    limit: mps_addr_t,
) -> mps_res_t {
    // This is the place where the magic happens, this relies on associated consts and
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
