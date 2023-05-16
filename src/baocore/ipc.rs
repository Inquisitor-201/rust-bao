use alloc::vec::Vec;
use spin::{Lazy, Mutex, RwLock};

use crate::{config::platform::qemu_aarch64_virt::linux_freertos::CONFIG, util::num_pages};

use super::{
    cpu::mycpu,
    mem::mem_alloc_ppages,
    types::{IrqID, Paddr, Vaddr},
};

#[derive(Clone)]
pub struct IPC {
    pub base: Vaddr,
    pub size: u64,
    pub shmem_id: usize,
    pub interrupts: Vec<IrqID>,
}

pub struct SharedMemConfig {
    pub size: u64,
}

pub struct SharedMem {
    pub size: u64,
    pub phys: Option<Paddr>,
    pub cpu_masters: Mutex<u64>,
}

impl SharedMem {
    pub fn new(size: u64) -> Self {
        Self {
            size,
            phys: None,
            cpu_masters: Mutex::new(0),
        }
    }
}

pub static SHMEM_LIST: Lazy<RwLock<Vec<SharedMem>>> = Lazy::new(|| RwLock::new(Vec::new()));

fn init_shmem_list() {
    let mut shmem_list = SHMEM_LIST.write();
    let shmem_configs = &CONFIG.read().shared_mem;
    for shmem_config in shmem_configs.iter() {
        shmem_list.push(SharedMem::new(shmem_config.size));
    }
}

fn alloc_shmem() {
    let mut shmem_list = SHMEM_LIST.write();
    for shmem in shmem_list.iter_mut() {
        if shmem.phys.is_none() {
            let n = num_pages(shmem.size as _);
            let ppages = mem_alloc_ppages(n, false).unwrap();
            assert!(ppages.num_pages == n);
            shmem.phys = Some(ppages.base);
        }
    }
}

pub fn init() {
    if mycpu().is_master() {
        init_shmem_list();
        alloc_shmem();
    }
}
