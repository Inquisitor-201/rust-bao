use lazy_static::lazy_static;
use spin::{Mutex, MutexGuard};

use crate::baocore::types::{AsType, Vaddr, MAX_VA, AsSecID};

pub const SEC_HYP_GLOBAL: AsSecID = 0;
pub const SEC_HYP_IMAGE: AsSecID = 1;
pub const SEC_HYP_PRIVATE: AsSecID = 2;
pub const SEC_HYP_VM: AsSecID = 3;
pub const SEC_VM_ANY: AsSecID = 0;

pub struct Section {
    pub id: AsSecID,
    pub is_hyp_sec: bool,
    pub beg: Vaddr,
    pub end: Vaddr,
    pub shared: bool,
}

extern "C" {
    static _dmem_beg: usize;
    static _cpu_private_beg: usize;
    static _cpu_private_end: usize;
    static _image_start: usize;
    static _image_end: usize;
    static _vm_beg: usize;
    static _vm_end: usize;
}

lazy_static! {
    static ref HYP_SECS: [Section; 4] = [
        Section {    // SEC_HYP_GLOBAL
            id: 0,
            is_hyp_sec: true,
            beg: unsafe { &_dmem_beg as *const _ as Vaddr },
            end: unsafe { &_cpu_private_beg as *const _ as Vaddr - 1 },
            shared: true,
        },
        Section {    // SEC_HYP_IMAGE
            id: 1,
            is_hyp_sec: true,
            beg: unsafe { &_image_start as *const _ as Vaddr },
            end: unsafe { &_image_end as *const _ as Vaddr - 1 },
            shared: true,
        },
        Section {    // SEC_HYP_PRIVATE
            id: 2,
            is_hyp_sec: true,
            beg: unsafe { &_cpu_private_beg as *const _ as Vaddr },
            end: unsafe { &_cpu_private_end as *const _ as Vaddr - 1 },
            shared: false,
        },
        Section {     // SEC_HYP_VM
            id: 3,
            is_hyp_sec: true,
            beg: unsafe { &_vm_beg as *const _ as Vaddr },
            end: unsafe { &_vm_beg as *const _ as Vaddr - 1 },
            shared: true,
        },
    ];
    static ref VM_SECS: [Section; 1] = [Section { // SEC_VM_ANY
        id: 0,
        is_hyp_sec: false,
        beg: 0x0,
        end: MAX_VA,
        shared: false,
    },];
    static ref HYP_SECS_LOCKS: [Mutex<()>; 4] = [Mutex::new(()), Mutex::new(()), Mutex::new(()), Mutex::new(())];
    static ref VM_SECS_LOCKS: [Mutex<()>; 1] = [Mutex::new(())];
}

pub struct Sections {
    pub sec: &'static [Section],
}

impl Section {
    pub fn lock(&self) -> MutexGuard<()> {
        if self.is_hyp_sec {
            HYP_SECS_LOCKS[self.id as usize].lock()
        } else {
            VM_SECS_LOCKS[self.id as usize].lock()
        }
    }
}

pub fn mem_get_sections(as_type: AsType) -> Sections {
    match as_type {
        AsType::AsHyp | AsType::AsHypCry => Sections {
            sec: HYP_SECS.as_ref(),
        },
        AsType::AsVM => Sections {
            sec: VM_SECS.as_ref(),
        },
    }
}
