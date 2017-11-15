use std::fmt;
use ffi::{mps_addr_t};

const  HANDLE_TABLE_SIZE: u64 = 5;

#[repr(C, packed)]
#[derive(Debug)]
pub struct Handle {
    index: u64
}

pub trait RootList<T> {
    fn alloc_handle(&mut self, item: T) -> Handle;
    fn free_handle(&mut self, handle : Handle);
}

pub union RootListItem {
    next : u64,
    content : mps_addr_t
}

impl fmt::Debug for RootListItem {
    fn fmt(&self, h: &mut fmt::Formatter) -> fmt::Result {
        write!(h, "{}", unsafe { self.next })
    }
}

#[derive(Debug)]
pub struct FreeRootItemList {
    openSlot: u64,
    freelist: Vec<RootListItem>
}

impl RootList<mps_addr_t> for FreeRootItemList {
    fn alloc_handle(&mut self, item: mps_addr_t) -> Handle {

        if self.openSlot == self.freelist.len() as u64 {
            panic!("No more Slots in RootList");
        }

        let next = unsafe { self.freelist[self.openSlot as usize].next };

        self.freelist[self.openSlot as usize].content =  item;

        let h = Handle {
            index: self.openSlot
        };

        self.openSlot = next;

        return h;
    }

    fn free_handle(&mut self, handle: Handle) {
        self.freelist[handle.index as usize].next = self.openSlot;
        self.openSlot = handle.index;
    }
}

#[derive(Debug)]
pub enum FreeItem<T> {
    next(u64),
    content(T)
}

impl<T> fmt::Display for FreeItem<T> where T : fmt::Display {
    fn fmt(&self, h: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FreeItem::next(x) => write!(h, "FreeItem({})",x ),
            FreeItem::content(ref y) => write!(h, "FreeItem({})", y)
        }
    }
}

pub fn buildHandleTable() -> FreeRootItemList {
    let mut freelist = Vec::new();

    for x in 1..HANDLE_TABLE_SIZE {
        freelist.push( RootListItem { next: x });
    }

    FreeRootItemList {
        openSlot: 0,
        freelist
    }
}

mod tests {

    use super::*;

    #[test]
    fn checkSlotAlloc() {
        let mut f : FreeRootItemList = buildHandleTable();
        let t  = f.alloc_handle(10000 as mps_addr_t);
        assert_eq!( unsafe { f.freelist[t.index as usize].content as u64 } , 10000);
    }

    #[test]
    fn handleAllocAndDrop() {
        let mut f : FreeRootItemList = buildHandleTable();
        let open = f.openSlot;
        let t  = f.alloc_handle(10000 as mps_addr_t);
        assert_eq!(f.openSlot, open);
    }

    #[test]
    fn handleAlloc() {
        let mut f : FreeRootItemList = buildHandleTable();
        let t = f.alloc_handle(10000 as mps_addr_t);
        assert_eq!(t.index, 1);
    }
}