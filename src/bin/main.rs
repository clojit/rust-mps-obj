extern crate memory_pool_system;

use std::fmt;
use memory_pool_system::ffi::{mps_addr_t};

const  HANDLE_TABLE_SIZE: u64 = 5;

#[repr(C, packed)]
#[derive(Debug)]
pub struct Handle {
    index: u64
}

impl fmt::Display for Handle {
    fn fmt(&self, h: &mut fmt::Formatter) -> fmt::Result {
        write!(h, "Handle({})", self.index)
    }
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
struct FreeRootItemList {
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


#[derive(Debug)]
struct FreeList<T> {
    openSlot: u64,
    freelist: Vec<FreeItem<T>>
}

fn buildHandleTable() -> FreeRootItemList {

    let mut freelist = Vec::new();

    for x in 1..HANDLE_TABLE_SIZE {
        freelist.push( RootListItem { next: x });
    }

    FreeRootItemList {
        openSlot: 0,
        freelist
    }
}

impl<T> RootList<T> for FreeList<T> {
    fn alloc_handle(&mut self, item: T) -> Handle {

        if self.openSlot == self.freelist.len() as u64 {
            panic!("No more Slots in RootList");
        }

        let newOpenSlot = {
            let ref newNext = self.freelist[self.openSlot as usize];

            match *newNext {
                FreeItem::next(openSlot) => openSlot,
                FreeItem::content(_) => {
                    panic!("MAJOR FUCKUP")
                }
            }
        };

        self.freelist[self.openSlot as usize] = FreeItem::content(item);

        let h = Handle {
            index: self.openSlot
        };

        self.openSlot = newOpenSlot;


        return h;
    }
    fn free_handle(&mut self, handle: Handle) {
        self.freelist[handle.index as usize] = FreeItem::next(self.openSlot);
        self.openSlot = handle.index;
    }
}

fn main() {

    let mut f : FreeRootItemList = buildHandleTable();

    println!("Old: {:?}", f);
    let t = f.alloc_handle(  10000 as mps_addr_t);
    println!("New: {:?}", f);
    println!("------------------------------");
    let t1 = f.alloc_handle( 10001 as mps_addr_t );
    println!("New 1: {:?}", f);
    println!("------------------------------");
    let t2 = f.alloc_handle( 10002 as mps_addr_t);
    println!("New 2: {:?}", f);
    println!("------------------------------");
    let t3 = f.alloc_handle( 10004 as mps_addr_t);
    println!("New 3: {:?}", f);
    println!("------------------------------");

    println!("Result: {:?}", t);
    println!("Result: {:?}", t1);
    println!("Result: {:?}", t2);
    println!("Result: {:?}", t3);

    println!("------------------------------");
    f.free_handle(t1);
    f.free_handle(t2);
    println!("New 5: {:?}", f);
    println!("------------------------------");

    let t4 = f.alloc_handle( 10005 as mps_addr_t );

    println!("New 6: {:?}", f);
    println!("New 3: {:?}", t4);
}