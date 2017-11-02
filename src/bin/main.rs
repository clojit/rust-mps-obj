use std::fmt;


const  HANDLE_TABLE_SIZE: u64 = 100;

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
    fn free_handle(&self, item: T) -> Handle;
}

#[derive(Debug)]
pub enum FreeItem<T> {
    next(u64),
    content(T)
}

impl<T> fmt::Display for FreeItem<T> {
    fn fmt(&self, h: &mut fmt::Formatter) -> fmt::Result {
        // Customize so only `x` and `y` are denoted.
        match *self {
            FreeItem::next(ref x) => write!(h, "FreeItem({})",x ),
            FreeItem::content(ref y) => write!(h, "FreeItem(content)")
        }

    }
}



#[derive(Debug)]
struct FreeList<T> {
    next: u64,
    freelist: Vec<FreeItem<T>>,
}

fn buildHandleTable<T>() -> FreeList<T> {

    let mut freelist = Vec::new();

    for x in 1..HANDLE_TABLE_SIZE {
        freelist.push( FreeItem::next(x) );
    }

    FreeList {
        next: 0,
        freelist,
    }
}

impl<T> RootList<T> for FreeList<T> {
    fn free_handle(&self, item : T) -> Handle {

        println!("self.next {}", self.next);

        let newNext = self.freelist[self.next];

        self.freelist[self.next] = item;

        println!("nextItem {}", nextItem);

        let h = Handle {
            index: self.next
        };

        self.next = newNext;

        return h;
    }
}

fn main() {
    let f : FreeList<u64> = buildHandleTable();
    let t = RootList::free_handle(&f, 5);
    //println!("{}", t);
}