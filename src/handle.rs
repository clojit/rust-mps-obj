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
    open_slot: u64,
    freelist: Vec<RootListItem>
}

impl RootList<mps_addr_t> for FreeRootItemList {
    fn alloc_handle(&mut self, item: mps_addr_t) -> Handle {

        if self.open_slot == self.freelist.len() as u64 {
            panic!("No more Slots in RootList");
        }

        let next = unsafe { self.freelist[self.open_slot as usize].next };

        self.freelist[self.open_slot as usize].content =  item;

        let h = Handle {
            index: self.open_slot
        };

        self.open_slot = next;

        return h;
    }

    fn free_handle(&mut self, handle: Handle) {
        self.freelist[handle.index as usize].next = self.open_slot;
        self.open_slot = handle.index;
    }
}

#[derive(Debug)]
pub enum FreeItem<T> {
    Next(u64),
    Content(T)
}

impl<T> fmt::Display for FreeItem<T> where T : fmt::Display {
    fn fmt(&self, h: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FreeItem::Next(x) => write!(h, "FreeItem({})",x ),
            FreeItem::Content(ref y) => write!(h, "FreeItem({})", y)
        }
    }
}

pub fn build_handle_table() -> FreeRootItemList {
    let mut freelist = Vec::new();

    for x in 1..HANDLE_TABLE_SIZE {
        freelist.push( RootListItem { next: x });
    }

    FreeRootItemList {
        open_slot: 0,
        freelist
    }
}

mod tests {

    use super::*;

    #[test]
    fn check_slot_alloc() {
        let mut f : FreeRootItemList = build_handle_table();
        let t  = f.alloc_handle(10000 as mps_addr_t);
        assert_eq!( unsafe { f.freelist[t.index as usize].content as u64 } , 10000);
    }

    #[test]
    fn handle_alloc_and_drop() {
        let mut f : FreeRootItemList = build_handle_table();
        let open = f.open_slot;
        let t  = f.alloc_handle(10000 as mps_addr_t);
        f.free_handle(t);
        assert_eq!(f.open_slot, open);
    }

    #[test]
    fn handle_alloc() {
        let mut f : FreeRootItemList = build_handle_table();
        let t = f.alloc_handle(10000 as mps_addr_t);
        assert_eq!(t.index, 0);
        let t2 = f.alloc_handle(10001 as mps_addr_t);
        assert_eq!(t2.index, 1);
    }
}