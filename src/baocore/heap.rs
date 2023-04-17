// use core::{sync::atomic::{AtomicBool, Ordering}, alloc::Layout, mem::size_of};

// use buddy_system_allocator::LockedHeap;

// use crate::arch::aarch64::armv8_a::cpu_arch_profile::CPU_MASTER;

// use super::types::CpuID;

// pub const HV_HEAP_SIZE: usize = 0x10000;

// static mut HEAP: [usize; HV_HEAP_SIZE / size_of::<usize>()] = [0; HV_HEAP_SIZE / size_of::<usize>()];

// #[cfg_attr(not(test), global_allocator)]
// static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::<32>::new();
// static HEAP_INIT_OK: AtomicBool = AtomicBool::new(false);

// #[alloc_error_handler]
// fn handle_alloc_error(layout: Layout) -> ! {
//     panic!("Heap allocation error, layout = {:?}", layout);
// }

// pub fn heap_init(cpu_id: CpuID) {
//     let heap_start = unsafe { HEAP.as_ptr() as usize };
//     unsafe {
//         HEAP_ALLOCATOR.lock().init(heap_start, HV_HEAP_SIZE);
//     }
// }
