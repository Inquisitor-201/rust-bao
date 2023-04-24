use aarch64::regs::PAR_EL1;
use spin::Mutex;
use tock_registers::interfaces::{Readable, Writeable};

use crate::{
    arch::aarch64::{
        armv8_a::{
            fences::{fence_sync, fence_sync_write},
            pagetable::{
                PageTableArch, HYP_PT_DSCR, PTE, PTE_HYP_FLAGS, PTE_INVALID, PTE_RSW_MSK,
                PTE_RSW_RSRV, PTE_TABLE, VM_PT_DSCR,
            },
        },
        defs::PAGE_SIZE,
        sysregs::{arm_at_s12e1w, arm_at_s1e2w, PAR_F, PAR_PA_MSK},
    },
    baocore::{
        cpu::mycpu,
        mem::{mem_alloc_page, mem_alloc_ppages, PPages},
        pagetable::{root_pt_addr, Pagetable},
        types::{AsSecID, AsType, Asid, ColorMap, MemFlags, Paddr, Vaddr, MAX_VA},
    },
    util::{clear_memory, is_aligned, num_pages, BaoError, BaoResult},
};

use super::sections::{mem_get_sections, SEC_HYP_PRIVATE, SEC_HYP_VM};

pub const HYP_ASID: u64 = 0;

#[repr(C)]
pub struct AddrSpace {
    pub pt: Pagetable,
    pub as_type: AsType,
    pub colors: ColorMap,
    pub id: Asid,
    pub lock: Mutex<()>,
}

pub trait AsArchTrait {
    fn arch_init(&mut self);
}

impl AddrSpace {
    pub fn init(
        &mut self,
        as_type: AsType,
        id: Asid,
        mut root_pt: Option<Vaddr>,
        colors: ColorMap,
    ) {
        self.as_type = as_type;
        self.colors = colors;
        self.id = id;
        self.lock = Mutex::new(());

        if root_pt.is_none() {
            self.pt.dscr = match as_type {
                AsType::AsVM => VM_PT_DSCR,
                _ => HYP_PT_DSCR,
            };
            let n = num_pages(self.pt.pt_size(0));
            let root = mem_alloc_page(
                n,
                match as_type {
                    AsType::AsVM => SEC_HYP_VM,
                    _ => SEC_HYP_PRIVATE,
                },
                true,
            )
            .unwrap();
            unsafe { *(root as *mut usize) = 99999; }
            clear_memory(root, n * PAGE_SIZE);
            root_pt = Some(root);
        }

        self.pt = Pagetable {
            root: root_pt.unwrap(),
            dscr: match as_type {
                AsType::AsVM => VM_PT_DSCR,
                _ => HYP_PT_DSCR,
            },
            arch: PageTableArch {
                rec_index: 0,
                rec_mask: 0,
            },
        };

        self.arch_init();
    }

    pub fn mem_find_sec(&self, va: Vaddr) -> Option<AsSecID> {
        let sections = mem_get_sections(self.as_type);
        sections
            .sec
            .iter()
            .enumerate()
            .find(|(_, section)| section.beg <= va && va <= section.end)
            .map(|x| x.0 as AsSecID)
    }

    pub fn mem_alloc_vpage(
        &mut self,
        section: AsSecID,
        at: Option<Vaddr>,
        n: usize,
    ) -> Option<Vaddr> {
        let mut lvl = 0;
        let mut count = 0;
        let mut failed = false;
        let sections = mem_get_sections(self.as_type);
        let sec = match sections.sec.get(section as usize) {
            Some(sec) => sec,
            None => return None,
        };

        let mut addr = match at {
            Some(at) => {
                if self
                    .mem_find_sec(at)
                    .map_or(true, |found_sec| found_sec != section)
                {
                    return None;
                }
                at
            }
            None => sec.beg,
        };

        let top = sec.end;

        if addr > top || !is_aligned(addr as usize, PAGE_SIZE) {
            return None;
        }

        let _sec_lock;
        let _as_lock = self.lock.lock();
        if sec.shared {
            _sec_lock = sec.lock();
        }

        let mut vpage = None;
        while count < n && !failed {
            // Check if there is still enough space in the address space.
            // The corner case of top being the highest address in the address
            // space and the target address being 0 is handled separate
            let full_as = addr == 0 && top == MAX_VA;
            if !full_as && ((top + 1 - addr) as usize / PAGE_SIZE) < n {
                vpage = None;
                failed = true;
                break;
            }

            let mut pte_ptr = self.pt.pt_get_pte(lvl, addr);
            let mut entry = self.pt.pt_getpteindex(pte_ptr, lvl);
            let nentries = self.pt.pt_nentries(lvl);
            let lvlsz = self.pt.pt_lvlsize(lvl);

            while entry < nentries && count < n && !failed {
                let pte = unsafe { *pte_ptr };

                if pte.check_rsw(PTE_RSW_RSRV) || pte.is_valid() && !pte.is_table(&self.pt, lvl) {
                    count = 0;
                    vpage = None;
                    if at.is_some() {
                        failed = true;
                        break;
                    }
                } else if !pte.is_valid() {
                    if pte.is_allocable(&self.pt, lvl, n - count, addr) {
                        if count == 0 {
                            vpage = Some(addr);
                        }
                        count += lvlsz / PAGE_SIZE;
                    } else {
                        self.alloc_pt_and_set(lvl, pte_ptr, addr);
                    }
                }

                if unsafe { *pte_ptr }.is_table(&self.pt, lvl) {
                    lvl += 1;
                    break;
                } else {
                    unsafe {
                        pte_ptr = pte_ptr.add(1);
                    }
                    entry += 1;
                    addr += lvlsz as u64;
                    if entry >= nentries {
                        lvl = 0;
                        break;
                    }
                }
            }
        }

        if vpage.is_some() && !failed {
            let mut count = 0;
            let mut addr = vpage.unwrap();
            let lvl = 0;
            while count < n {
                let mut pte_ptr = None;
                for lvl in 0..self.pt.dscr.lvls {
                    let p = self.pt.pt_get_pte(lvl, addr);
                    if !unsafe { *p }.is_valid() {
                        pte_ptr = Some(p);
                        break;
                    }
                }
                assert!(pte_ptr.is_some());
                unsafe { *pte_ptr.unwrap() }.set_rsw(PTE_RSW_RSRV);
                let lvlsize = self.pt.pt_lvlsize(lvl);
                addr += lvlsize as u64;
                count += lvlsize / PAGE_SIZE;
            }
        }

        vpage
    }

    pub fn mem_map(
        &self,
        va: Vaddr,
        ppages: Option<&PPages>,
        num_pages: usize,
        flags: MemFlags,
    ) -> BaoResult<Vaddr> {
        assert!(is_aligned(va as usize, PAGE_SIZE));
        let mut count = 0;
        let mut pte_ptr = None;
        let mut vaddr = va;

        let sec = self.mem_find_sec(vaddr);

        if sec.is_none() || sec != self.mem_find_sec(vaddr + (num_pages * PAGE_SIZE) as u64 - 1) {
            return Err(BaoError::NotFound);
        }

        let sec = mem_get_sections(self.as_type)
            .sec
            .get(sec.unwrap() as usize)
            .unwrap();

        let _sec_lock;
        let _as_lock = self.lock.lock();
        if sec.shared {
            _sec_lock = sec.lock();
        }

        // todo: colors

        let mut paddr = ppages.as_ref().map_or(0, |ppages| ppages.base);
        while count < num_pages {
            let mut lvl = 0;
            for _ in 0..self.pt.dscr.lvls {
                pte_ptr = Some(self.pt.pt_get_pte(lvl, vaddr));
                if self.pt.dscr.lvl_term[lvl] {
                    let pte = unsafe { *pte_ptr.unwrap() };
                    if pte.is_mappable(&self.pt, lvl, num_pages - count, vaddr, paddr) {
                        break;
                    } else if !pte.is_valid() {
                        self.alloc_pt_and_set(lvl, pte_ptr.unwrap(), vaddr);
                        // self.mem_alloc_pt(pte, lvl, vaddr);
                    } else if !pte.is_table(&self.pt, lvl) {
                        panic!("trying to override previous mapping");
                    }
                }
                lvl += 1;
            }

            let mut entry = self.pt.pt_getpteindex(pte_ptr.unwrap(), lvl);
            let nentries = self.pt.pt_nentries(lvl);
            let lvlsz = self.pt.pt_lvlsize(lvl);

            while entry < nentries && count < num_pages && num_pages - count >= lvlsz / PAGE_SIZE {
                if ppages.is_none() {
                    match mem_alloc_ppages(lvlsz / PAGE_SIZE, true) {
                        Some(ppages) => {
                            paddr = ppages.base;
                        }
                        None => return Err(BaoError::OutOfMemory),
                    }
                }
                unsafe {
                    *(pte_ptr.unwrap()) = PTE::new(paddr, self.pt.page_type(lvl), flags);
                }
                vaddr += lvlsz as u64;
                paddr += lvlsz as u64;
                count += lvlsz / PAGE_SIZE;
                entry += 1;
                pte_ptr = Some(unsafe { pte_ptr.unwrap().add(1) });
            }
        }

        fence_sync();
        Ok(va)
    }

    pub fn mem_alloc_map(
        &mut self,
        section: AsSecID,
        ppages: Option<&PPages>,
        at: Option<Vaddr>,
        num_pages: usize,
        flags: MemFlags,
    ) -> BaoResult<Vaddr> {
        match self.mem_alloc_vpage(section, at, num_pages) {
            Some(va) =>  {
                let r = self.mem_map(va, ppages, num_pages, flags);
                r
            }
            _ => Err(BaoError::NotFound),
        }
    }

    pub fn mem_alloc_map_dev(
        &mut self,
        section: AsSecID,
        pa: Paddr,
        at: Option<Vaddr>,
        num_pages: usize,
    ) -> BaoResult<Vaddr> {
        assert!(is_aligned(pa as _, PAGE_SIZE));
        match self.mem_alloc_vpage(section, at, num_pages) {
            Some(va) => {
                let flags = match self.as_type {
                    AsType::AsHyp => PTE_HYP_FLAGS,
                    _ => todo!("PTE_VM_DEV_FLAGS"),
                };
                let ppages = PPages::new(pa, num_pages);
                self.mem_map(va, Some(&ppages), num_pages, flags)
            }
            _ => Err(BaoError::NotFound),
        }
    }

    pub fn mem_translate(&self, va: Vaddr) -> Option<Paddr> {
        let par_saved = PAR_EL1.get();
        let par = match self.as_type {
            AsType::AsHyp | AsType::AsHypCry => arm_at_s1e2w(va),
            AsType::AsVM => arm_at_s12e1w(va),
        };
        PAR_EL1.set(par_saved);
        if par & PAR_F != 0 {
            None
        } else {
            Some((par & PAR_PA_MSK) | (va & (PAGE_SIZE as u64 - 1)))
        }
    }

    pub fn alloc_pt_and_set(&self, lvl: usize, pte_ptr: *mut PTE, vaddr: Vaddr) {
        let pt_size = num_pages(self.pt.pt_size(lvl + 1));
        if pt_size > 1 {
            unimplemented!("alloc_pt_and_set: pt is too big")
        }
        match mem_alloc_ppages(pt_size, false) {
            Some(ppages) => {
                let pte_dflt_val = PTE_INVALID | (unsafe { *pte_ptr }.0 & PTE_RSW_MSK);
                unsafe { *pte_ptr = PTE::new(ppages.base, PTE_TABLE, PTE_HYP_FLAGS) }
                fence_sync_write();
                let temp_pt = self.pt.pt_get(lvl + 1, vaddr);
                for i in 0..self.pt.pt_nentries(lvl + 1) {
                    unsafe {
                        (*temp_pt.add(i)).0 = pte_dflt_val;
                    }
                }
            }
            None => panic!("alloc_pt_and_set: no free mem"),
        }
    }
}

pub fn mem_prot_init() {
    let root_pt = root_pt_addr();
    mycpu()
        .addr_space
        .init(AsType::AsHyp, HYP_ASID, Some(root_pt), 0);
}
