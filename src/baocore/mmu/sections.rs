use spin::{Mutex, MutexGuard, Once};

use crate::{
    arch::aarch64::defs::{BAO_CPU_BASE, BAO_VAS_TOP, BAO_VM_BASE},
    baocore::types::{AsSecID, AsType, Vaddr, MAX_VA},
};

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
    static _image_start: usize;
    static _image_end: usize;
}

pub static HYP_SECS: Once<[Section; 4]> = Once::new();
pub static VM_SECS: [Section; 1] = [Section {
    // SEC_VM_ANY
    id: 0,
    is_hyp_sec: false,
    beg: 0x0,
    end: MAX_VA,
    shared: false,
}];
pub static HYP_SECS_LOCKS: [Mutex<()>; 4] = [
    Mutex::new(()),
    Mutex::new(()),
    Mutex::new(()),
    Mutex::new(()),
];
pub static VM_SECS_LOCKS: [Mutex<()>; 1] = [Mutex::new(())];

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
            sec: HYP_SECS.call_once(|| {
                [
                    Section {
                        // SEC_HYP_GLOBAL
                        id: 0,
                        is_hyp_sec: true,
                        beg: unsafe { &_dmem_beg as *const _ as Vaddr },
                        end: BAO_CPU_BASE as Vaddr - 1,
                        shared: true,
                    },
                    Section {
                        // SEC_HYP_IMAGE
                        id: 1,
                        is_hyp_sec: true,
                        beg: unsafe { &_image_start as *const _ as Vaddr },
                        end: unsafe { &_image_end as *const _ as Vaddr - 1 },
                        shared: true,
                    },
                    Section {
                        // SEC_HYP_PRIVATE
                        id: 2,
                        is_hyp_sec: true,
                        beg: BAO_CPU_BASE as Vaddr,
                        end: BAO_VM_BASE as Vaddr - 1,
                        shared: false,
                    },
                    Section {
                        // SEC_HYP_VM
                        id: 3,
                        is_hyp_sec: true,
                        beg: BAO_VM_BASE as Vaddr,
                        end: BAO_VAS_TOP as Vaddr - 1,
                        shared: true,
                    },
                ]
            }),
        },
        AsType::AsVM => Sections {
            sec: VM_SECS.as_ref(),
        },
    }
}
