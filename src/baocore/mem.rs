use core::mem::MaybeUninit;

use spin::{Lazy, Mutex};

use super::{
    cpu::{mem_cpu_boot_alloc_size, mycpu, CPU_SYNC_TOKEN},
    heap,
    mmu::{mem::mem_prot_init, sections::SEC_HYP_GLOBAL},
    types::{AsSecID, ColorMap, Paddr},
};
use crate::{
    arch::aarch64::{armv8_a::pagetable::PTE_HYP_FLAGS, defs::PAGE_SIZE},
    config,
    platform::PLATFORM,
    util::{
        bitmap::Bitmap, image_load_size, image_noload_size, image_size, num_pages, range_in_range,
        vm_image_size, BaoError, BaoResult,
    },
};

pub const MAX_PAGE_POOLS: usize = 4;

pub struct PagePools {
    pools: [Option<&'static mut MemPagePool>; MAX_PAGE_POOLS],
}

#[allow(invalid_value)]
pub static PAGE_POOLS: Lazy<Mutex<PagePools>> = Lazy::new(|| {
    Mutex::new(PagePools {
        pools: {
            let mut pools: [Option<&'static mut MemPagePool>; MAX_PAGE_POOLS] =
                unsafe { MaybeUninit::uninit().assume_init() };
            for i in 0..MAX_PAGE_POOLS {
                pools[i] = None;
            }
            pools
        },
    })
});

impl PagePools {
    pub fn insert(&mut self, pool: &'static mut MemPagePool) {
        for p in self.pools.iter_mut() {
            if p.is_none() {
                *p = Some(pool);
                return;
            }
        }
        panic!("can't find free slot in PagePools");
    }
}

pub fn add_page_pool(pool: &'static mut MemPagePool) {
    PAGE_POOLS.lock().insert(pool);
}

pub fn mem_alloc_ppages(num_pages: usize, aligned: bool) -> Option<PPages> {
    let mut r = PAGE_POOLS.lock();
    for pp in r.pools.iter_mut() {
        if let Some(pp) = pp {
            let ppages = pp.alloc(num_pages, aligned);
            if ppages.is_some() {
                return ppages;
            }
        };
    }
    None
}

pub fn mem_alloc_page(num_pages: usize, sec: AsSecID, phys_aligned: bool) -> Result<u64, BaoError> {
    if let Some(ppages) = mem_alloc_ppages(num_pages, phys_aligned) {
        if ppages.num_pages == num_pages {
            return mycpu().addr_space.mem_alloc_map(
                sec,
                Some(&ppages),
                None,
                num_pages,
                PTE_HYP_FLAGS,
            );
        }
    }
    Err(BaoError::OutOfMemory)
}

#[repr(C)]
pub struct PPages {
    pub base: Paddr,
    pub num_pages: usize,
    pub colors: ColorMap,
}

impl PPages {
    pub fn new(base: Paddr, n: usize) -> Self {
        Self {
            base: base,
            num_pages: n,
            colors: 0,
        }
    }
}

#[repr(C)]
pub struct MemPagePool {
    base: Paddr,
    size: usize,
    free: usize,
    last: usize,
    bitmap: Option<Bitmap>,
    lock: Mutex<()>,
}

impl MemPagePool {
    pub fn set_up_bitmap(&mut self, load_addr: Paddr) -> BaoResult<()> {
        let cpu_size = PLATFORM.cpu_num * mem_cpu_boot_alloc_size();
        let bitmap_num_pages = self.size.div_ceil(8 * PAGE_SIZE);
        let bitmap_base =
            load_addr + image_size() as u64 + vm_image_size() as u64 + cpu_size as u64;
        let mut bitmap_pp = PPages::new(bitmap_base, bitmap_num_pages);
        let root_bitmap = mycpu().addr_space.mem_alloc_map(
            SEC_HYP_GLOBAL,
            Some(&mut bitmap_pp),
            None,
            bitmap_num_pages,
            PTE_HYP_FLAGS,
        )?;
        self.bitmap = Some(Bitmap::new(root_bitmap, bitmap_num_pages * PAGE_SIZE));
        self.bitmap.as_mut().unwrap().clear_all();
        if !self.reserve_ppages(&bitmap_pp) {
            return Err(BaoError::AlreadyExists);
        }
        Ok(())
    }

    pub fn reserve_ppages(&mut self, ppages: &PPages) -> bool {
        let is_in_rgn = range_in_range(
            ppages.base as usize,
            ppages.num_pages * PAGE_SIZE,
            self.base as usize,
            self.size * PAGE_SIZE,
        );
        if !is_in_rgn {
            return true;
        }

        let pageoff = num_pages((ppages.base - self.base) as _);
        let was_free = !self.are_ppages_reserved(ppages);
        if was_free {
            self.bitmap
                .as_mut()
                .unwrap()
                .set_consecutive(pageoff, ppages.num_pages);
            self.free -= ppages.num_pages;
        }
        was_free
    }

    pub fn are_ppages_reserved(&self, ppages: &PPages) -> bool {
        let rgn_found = range_in_range(
            ppages.base as _,
            ppages.num_pages * PAGE_SIZE,
            self.base as _,
            self.size * PAGE_SIZE,
        );

        if rgn_found {
            let pageoff = num_pages((ppages.base - self.base) as _);

            // verify these pages arent allocated yet
            let is_alloced = self.bitmap.as_ref().unwrap().get(pageoff);
            let avlbl_contig_pp = self
                .bitmap
                .as_ref()
                .unwrap()
                .count_consecutive(pageoff, ppages.num_pages);

            if is_alloced || avlbl_contig_pp < ppages.num_pages {
                return true;
            }
        }

        false
    }

    pub fn reserve_hyp_mem(&mut self, load_addr: Paddr) -> BaoResult<()> {
        let image_load_size = image_load_size();
        let image_noload_size = image_noload_size();
        let vm_image_size = vm_image_size();
        let cpu_size = PLATFORM.cpu_num * mem_cpu_boot_alloc_size();

        let image_noload_addr = load_addr + image_load_size as u64 + vm_image_size as u64;
        let cpu_base_addr = image_noload_addr + image_noload_size as u64;

        let images_load_ppages = PPages::new(load_addr, num_pages(image_load_size));
        let images_noload_ppages = PPages::new(image_noload_addr, num_pages(image_noload_size));
        let cpu_ppages = PPages::new(cpu_base_addr, num_pages(cpu_size));

        let image_load_reserved = self.reserve_ppages(&images_load_ppages);
        let image_noload_reserved = self.reserve_ppages(&images_noload_ppages);
        let cpu_reserved = self.reserve_ppages(&cpu_ppages);

        if image_load_reserved && image_noload_reserved && cpu_reserved {
            Ok(())
        } else {
            Err(BaoError::AlreadyExists)
        }
    }

    pub fn alloc(&mut self, num_pages: usize, aligned: bool) -> Option<PPages> {
        assert!(!aligned);
        if self.free < num_pages {
            return None;
        }
        if num_pages == 0 {
            return Some(PPages::new(0, 0));
        }
        let _lock = self.lock.lock();

        let mut curr = self.last;
        let bitmap = self.bitmap.as_mut().unwrap();
        for _ in 0..2 {
            match bitmap.find_consec(curr, num_pages, false) {
                Some(bit) => {
                    let p = PPages::new(self.base + (bit * PAGE_SIZE) as u64, num_pages);
                    bitmap.set_consecutive(bit, num_pages);
                    self.free -= num_pages;
                    self.last = bit + num_pages;
                    return Some(p);
                }
                None => curr = 0,
            }
        }
        None
    }
}

#[repr(C)]
pub struct MemRegion {
    base: Paddr,
    size: usize,
    page_pool: MemPagePool,
}

impl MemRegion {
    pub const fn new(base: Paddr, size: usize) -> Self {
        MemRegion {
            base,
            size,
            page_pool: MemPagePool {
                base: 0,
                size: 0,
                free: 0,
                last: 0,
                bitmap: None,
                lock: Mutex::new(()),
            },
        }
    }

    pub fn page_pool_root_init(&mut self, load_addr: Paddr) -> BaoResult<()> {
        let pool_sz = self.size / PAGE_SIZE;
        let mut page_pool = MemPagePool {
            base: self.base,
            size: pool_sz,
            free: pool_sz,
            last: 0,
            bitmap: None,
            lock: Mutex::new(()),
        };
        page_pool.set_up_bitmap(load_addr)?;
        page_pool.reserve_hyp_mem(load_addr)?;
        self.page_pool = page_pool;
        Ok(())
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
    root_mem_region.page_pool_root_init(load_addr)?;
    Ok(root_mem_region)
}

pub fn init(load_addr: Paddr) {
    mem_prot_init();
    if mycpu().is_master() {
        // todo: cache_arch_enumerate()
        let mem_region = match mem_setup_root_pool(load_addr) {
            Ok(m) => m,
            Err(e) => panic!("{:#x?}", e),
        };
        add_page_pool(&mut mem_region.page_pool);
        heap::init();
        config::init(load_addr);
    }
    CPU_SYNC_TOKEN.sync_and_clear_msg();
}
