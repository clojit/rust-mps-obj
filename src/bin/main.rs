extern crate memory_pool_system;

use memory_pool_system::handle::{FreeRootItemList, buildHandleTable, RootList};
use memory_pool_system::ffi::mps_addr_t;


fn main() {

    let mut f : FreeRootItemList = buildHandleTable();

    println!("Old: {:?}", f);
    let t = f.alloc_handle(  10000 as mps_addr_t);
    println!("New: {:?}", f);
    println!("------------------------------");
    let t1 = f.alloc_handle( 10001 as mps_addr_t);
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