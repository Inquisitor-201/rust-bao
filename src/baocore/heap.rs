use crate::{
    arch::aarch64::{armv8_a::pagetable::PTE_HYP_FLAGS, defs::PAGE_SIZE},
    baocore::{cpu::mycpu, mmu::sections::SEC_HYP_GLOBAL},
};
use alloc::vec::Vec;
use buddy_system_allocator::LockedHeap;
use core::alloc::Layout;

// use super::types::CpuID;

pub const HV_HEAP_SIZE: usize = 0x20000;

#[cfg_attr(not(test), global_allocator)]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::<32>::new();

#[alloc_error_handler]
fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

fn simple_test() {
    let mut vec: Vec<i32> = Vec::with_capacity(HV_HEAP_SIZE / 0x20);
    assert_eq!(vec.capacity(), HV_HEAP_SIZE / 0x20);
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(4);
    vec.push(5);
    assert_eq!(vec, [1, 2, 3, 4, 5]);
}

pub fn init() {
    assert!(mycpu().is_master());
    let heap_start = mycpu()
        .addr_space
        .mem_alloc_map(
            SEC_HYP_GLOBAL,
            None,
            None,
            HV_HEAP_SIZE / PAGE_SIZE,
            PTE_HYP_FLAGS,
        )
        .unwrap();
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(heap_start as usize, HV_HEAP_SIZE);
    }
    simple_test();
}
