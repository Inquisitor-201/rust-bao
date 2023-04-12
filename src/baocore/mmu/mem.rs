use aarch64::regs::PAR_EL1;
use spin::Mutex;
use tock_registers::interfaces::{Readable, Writeable};

use crate::{
    arch::aarch64::{
        armv8_a::pagetable::{PageTableArch, HYP_PT_DSCR, VM_PT_DSCR},
        defs::PAGE_SIZE,
        sysregs::{arm_at_s12e1w, arm_at_s1e2w, PAR_F, PAR_PA_MSK},
    },
    baocore::{
        cpu::mycpu,
        mem::PPages,
        pagetable::{root_pt_addr, Pagetable},
        types::{AsSecID, AsType, Asid, ColorMap, MemFlags, Paddr, Vaddr, MAX_VA},
    },
    util::{is_aligned, BaoError, BaoResult},
};

use super::sections::mem_get_sections;

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
    pub fn init(&mut self, as_type: AsType, id: Asid, root_pt: Vaddr, colors: ColorMap) {
        self.as_type = as_type;
        self.colors = colors;
        self.id = id;

        if root_pt == 0 {
            todo!();
        }

        self.pt = Pagetable {
            root: root_pt,
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
    ) -> BaoResult<Vaddr> {
        let mut count = 0;
        let mut failed = false;

        let sections = mem_get_sections(self.as_type);
        let sec = match sections.sec.get(section as usize) {
            Some(sec) => sec,
            None => return Err(BaoError::InvalidParam),
        };

        let addr = match at {
            Some(at) => {
                if self
                    .mem_find_sec(at)
                    .map_or(true, |found_sec| found_sec != section)
                {
                    return Err(BaoError::InvalidParam);
                }
                at
            }
            None => sec.beg,
        };

        let top = sec.end;

        if addr > top || !is_aligned(addr as usize, PAGE_SIZE) {
            return Err(BaoError::InvalidParam);
        }

        let _sec_lock;
        let _as_lock = self.lock.lock();
        if sec.shared {
            _sec_lock = sec.lock();
        }

        let mut vpage;
        while count < n && !failed {
            // Check if there is still enough space in the address space.
            // The corner case of top being the highest address in the address
            // space and the target address being 0 is handled separate
            let full_as = addr == 0 && top == MAX_VA;
            if !full_as && ((top + 1 - addr) as usize / PAGE_SIZE) < n {
                vpage = usize::MAX;
                failed = true;
                break;
            }

            // pte = Some(pt_get_pte(self.pt, lvl, addr));
            // entry = pt_getpteindex(&as.pt, pte.unwrap(), lvl);
            // nentries = pt_nentries(&as.pt, lvl);
            // lvlsze = pt_lvlsize(&as.pt, lvl);

            //     while (entry < nentries) && (count < n) && !failed {
            //         if pte_check_rsw(pte.unwrap(), PTE_RSW_RSRV) || (pte_valid(pte.unwrap()) && !pte_table(&as.pt, pte.unwrap(), lvl)) {
            //             count = 0;
            //             vpage = INVALID_VA;
            //             if at != INVALID_VA {
            //                 failed = true;
            //                 break;
            //             }
            //         } else if !pte_valid(pte.unwrap()) {
            //             if pte_allocable(as, pte.unwrap(), lvl, n - count, addr) {
            //                 if count == 0 {
            //                     vpage = addr;
            //                 }
            //                 count += lvlsze / PAGE_SIZE;
            //             } else {
            //                 if mem_alloc_pt(as, pte.unwrap(), lvl, addr).is_null() {
            //                     ERROR("failed to alloc page table");
            //                 }
            //             }
            //         }

            //         if pte_table(&as.pt, pte.unwrap(), lvl) {
            //             lvl += 1;
            //             break;
            //         } else {
            //             pte = Some(unsafe { pte.unwrap().add(1) });
            //             addr += lvlsze;
            //             if entry + 1 >= nentries {
            //                 lvl = 0;
            //                 break;
            //             }
            //             entry += 1;
            //         }
            //     }
            // }

            // if vpage != INVALID_VA && !failed {
            //     count = 0;
            //     addr = vpage;
            //     let mut lvl = 0;
            //     while count < n {
            //         for lvl in 0..as.pt.dscr.lvls {
            //             pte = Some(pt_get_pte(&mut as.pt, lvl, addr));
            //             if !pte_valid(pte.unwrap()) {
            //                 break;
            //             }
            //         }
            //         pte_set_rsw(pte.unwrap(), PTE_RSW_RSRV);
            //         addr += pt_lvlsize(&as.pt, lvl);
            //         count += pt_lvlsize(&as.pt, lvl) / PAGE_SIZE;
            //     }
            // }

            // if sec.shared {
            //     spin_unlock(&sec.lock);
            // }

            // spin_unlock(&as.lock);

            // vpage
        }
        todo!()
    }

    pub fn mem_alloc_map(
        &mut self,
        section: AsSecID,
        ppages: Option<&mut PPages>,
        at: Option<Vaddr>,
        num_pages: usize,
        flags: MemFlags,
    ) -> BaoResult<Vaddr> {
        self.mem_alloc_vpage(section, at, num_pages);
        loop {}
        // let sections =
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
}

pub fn mem_prot_init() {
    let root_pt = root_pt_addr();
    mycpu().addr_space.init(AsType::AsHyp, HYP_ASID, root_pt, 0);
}
