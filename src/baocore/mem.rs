use core::mem::{size_of, MaybeUninit};

use super::{
    cpu::{cpu, Cpu},
    mmu::mem::mem_prot_init,
    types::{ColorMap, Paddr},
};
use crate::{
    arch::aarch64::defs::PAGE_SIZE,
    platform::PLATFORM,
    util::{image_size, range_in_range, vm_image_size, BaoError, BaoResult},
};

#[repr(C)]
pub struct PPages {
    base: Paddr,
    num_pages: usize,
    colors: ColorMap,
}

impl PPages {
    pub fn mem_ppages_get(base: Paddr, n: usize) -> Self {
        Self {
            base: base,
            num_pages: n,
            colors: 0,
        }
    }
}

#[repr(C)]
pub struct MemPagePool {
    // node: node_t,
    base: Paddr,
    size: usize,
    free: usize,
    last: usize,
    // bitmap: *mut bitmap_t,
    // lock: spinlock_t,
}

impl MemPagePool {
    pub fn set_up_bitmap(&self, load_addr: Paddr) -> BaoResult<()> {
        let cpu_size =
            PLATFORM.cpu_num * (size_of::<Cpu>() + cpu().addr_space.pt.dscr.lvls * PAGE_SIZE);
        let bitmap_num_pages = self.size.div_ceil(8 * PAGE_SIZE);
        let bitmap_base =
            load_addr + image_size() as u64 + vm_image_size() as u64 + cpu_size as u64;
        let bitmap_pp = PPages::mem_ppages_get(bitmap_base, bitmap_num_pages);
    }
}

#[repr(C)]
pub struct MemRegion {
    base: Paddr,
    size: usize,
    page_pool: MaybeUninit<MemPagePool>,
}

impl MemRegion {
    pub const fn new(base: Paddr, size: usize) -> Self {
        MemRegion {
            base,
            size,
            page_pool: MaybeUninit::uninit(),
        }
    }

    pub fn page_pool_root_init(&mut self, load_addr: Paddr) -> BaoResult<()> {
        let pool_sz = self.size / PAGE_SIZE;
        let mut page_pool = MemPagePool {
            base: self.base,
            size: pool_sz,
            free: pool_sz,
            last: 0,
        };
        page_pool.set_up_bitmap()?;
        page_pool.reserve_hyp_mem()?;
        self.page_pool.write(page_pool);
    }
}

fn mem_find_root_region(load_addr: Paddr) -> BaoResult<&'static mut MemRegion> {
    let image_size = image_size();

    /* Find the root memory region in which the hypervisor was loaded. */
    for i in 0..PLATFORM.region_num {
        let region = unsafe { &mut *(&PLATFORM.regions[i] as *const _ as *mut MemRegion) };
        let is_in_rgn = range_in_range(load_addr as _, image_size, region.base as _, region.size);
        if is_in_rgn {
            return Ok(region);
        }
    }

    Err(BaoError::NotFound)
}

fn mem_setup_root_pool(load_addr: Paddr) -> BaoResult<&'static mut MemRegion> {
    let root_mem_region = mem_find_root_region(load_addr)?;
    root_mem_region.page_pool_root_init()?;
}

pub fn init(load_addr: Paddr) {
    mem_prot_init();
    if cpu().is_master() {
        // todo: cache_arch_enumerate()
        mem_setup_root_pool(load_addr)
    }
}
