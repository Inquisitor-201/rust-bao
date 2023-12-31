use crate::{arch::aarch64::defs::PAGE_SIZE, baocore::types::Vaddr, util::is_aligned};

#[repr(C)]
pub struct Bitmap {
    base: Vaddr,
    size_bytes: usize,
}

#[repr(C)]
#[repr(align(0x1000))]
pub struct BMSpace(pub [u8; PAGE_SIZE]);

impl BMSpace {
    pub fn base(&self) -> Vaddr {
        &self.0[0] as *const _ as _
    }
}

impl Bitmap {
    pub fn new(base: Vaddr, size_bytes: usize) -> Self {
        assert!(is_aligned(base as usize, PAGE_SIZE));
        Self { base, size_bytes }
    }

    pub fn clear_all(&mut self) {
        let ptr = self.base as *mut u8;
        unsafe {
            core::ptr::write_bytes(ptr, 0, self.size_bytes);
        }
    }

    pub fn get(&self, index: usize) -> bool {
        assert!(index < self.size_bytes * 8); // make sure index is in bounds

        let byte_index = index / 8;
        let bit_index = index % 8;
        let byte_ptr = (self.base as usize + byte_index) as *const u8;
        let byte = unsafe { *byte_ptr };

        ((byte >> bit_index) & 1) == 1
    }

    pub fn set(&mut self, index: usize) {
        assert!(index < self.size_bytes * 8); // make sure index is in bounds

        let byte_index = index / 8;
        let bit_index = index % 8;
        let byte_ptr = (self.base as usize + byte_index) as *mut u8;

        unsafe { *byte_ptr |= 1 << bit_index }
    }

    pub fn clear(&mut self, index: usize) {
        assert!(index < self.size_bytes * 8); // make sure index is in bounds

        let byte_index = index / 8;
        let bit_index = index % 8;
        let byte_ptr = (self.base as usize + byte_index) as *mut u8;

        unsafe { *byte_ptr &= !(1 << bit_index) }
    }

    pub fn count_consecutive(&self, from: usize, num_bits: usize) -> usize {
        assert!(from < self.size_bytes * 8);
        let mut count = 0;
        let b = self.get(from);

        for i in from..(from + num_bits).min(self.size_bytes * 8) {
            if self.get(i) == b {
                count += 1;
            } else {
                break;
            }
            if count == num_bits {
                break;
            }
        }

        count
    }

    pub fn clear_consecutive(&mut self, start: usize, n: usize) {
        for i in 0..n {
            self.clear(start + i);
        }
    }

    pub fn set_consecutive(&mut self, start: usize, n: usize) {
        for i in 0..n {
            self.set(start + i);
        }
    }

    pub fn find_consec(&self, start: usize, n: usize, set: bool) -> Option<usize> {
        assert!(start < self.size_bytes * 8);
        let mut consec_count = 0;
        let mut consec_start = None;

        for i in start..(self.size_bytes * 8) {
            if self.get(i) == set {
                if consec_count == 0 {
                    consec_start = Some(i);
                }
                consec_count += 1;
            } else {
                consec_count = 0;
                consec_start = None;
            }

            if consec_count == n {
                break;
            }
        }

        if consec_count == n {
            consec_start
        } else {
            None
        }
    }
}
