pub mod bitmap;

use crate::{arch::aarch64::defs::PAGE_SIZE, baocore::types::Vaddr};

#[inline]
pub fn range_in_range(base1: usize, size1: usize, base2: usize, size2: usize) -> bool {
    let limit1 = if let Some(val) = base1.checked_add(size1) {
        val
    } else {
        usize::MAX
    };
    let limit2 = if let Some(val) = base2.checked_add(size2) {
        val
    } else {
        usize::MAX
    };

    (base1 >= base2) && (limit1 <= limit2)
}

pub fn image_size() -> usize {
    extern "C" {
        static _image_start: usize;
        static _image_end: usize;
    }
    unsafe { &_image_end as *const _ as usize - &_image_start as *const _ as usize }
}

pub fn vm_image_size() -> usize {
    extern "C" {
        static _vm_image_start: usize;
        static _vm_image_end: usize;
    }
    unsafe { &_vm_image_end as *const _ as usize - &_vm_image_start as *const _ as usize }
}

pub fn image_load_size() -> usize {
    extern "C" {
        static _image_load_end: usize;
        static _image_start: usize;
    }
    unsafe { &_image_load_end as *const _ as usize - &_image_start as *const _ as usize }
}

pub fn image_noload_size() -> usize {
    extern "C" {
        static _image_end: usize;
        static _image_load_end: usize;
    }
    unsafe { &_image_end as *const _ as usize - &_image_load_end as *const _ as usize }
}

pub fn align(val: usize, to: usize) -> usize {
    val.div_ceil(to) * to
}

pub fn is_aligned(val: usize, to: usize) -> bool {
    val % to == 0
}

pub fn align_floor(val: usize, to: usize) -> usize {
    val / to * to
}

pub fn num_pages(sz: usize) -> usize {
    sz.div_ceil(PAGE_SIZE)
}

pub fn clear_memory(va: Vaddr, sz: usize) {
    for addr in va..va + sz as u64 {
        unsafe {
            (addr as *mut u8).write_volatile(0);
        }
    }
}

pub const PAGE_OFFSET_MASK: usize = PAGE_SIZE - 1;
pub const PAGE_FRAME_MASK: usize = !PAGE_OFFSET_MASK;

#[derive(Debug)]
pub enum BaoError {
    AlreadyExists,
    BadState,
    InvalidParam,
    NotFound,
    OutOfMemory,
    ResourceBusy,
    Unsupported,
}

pub type BaoResult<T> = Result<T, BaoError>;
