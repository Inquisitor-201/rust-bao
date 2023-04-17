use core::mem::size_of;

use spin::RwLock;

use crate::{arch::aarch64::{
    armv8_a::cpu_arch_profile::CPU_MASTER,
    cpu::CpuArch,
    defs::{BAO_CPU_BASE, CPU_STACK_SIZE, PAGE_SIZE},
}, util::align, platform::PLATFORM};

use super::{
    mmu::mem::AddrSpace,
    types::{CpuID, Paddr},
};

#[repr(C)]
#[repr(align(0x1000))]
pub struct CpuStack {
    stack: [u8; CPU_STACK_SIZE],
}

#[repr(C)]
pub struct Cpu {
    pub id: CpuID,
    pub handling_msgs: bool,
    pub addr_space: AddrSpace,
    // vcpu: *mut Vcpu,
    pub arch: CpuArch,
    // interface: *mut CpuIf,
    stack: CpuStack,
}

impl Cpu {
    pub fn is_master(&self) -> bool {
        self.id == unsafe { CPU_MASTER }
    }
}

pub trait CpuArchTrait {
    fn arch_init(&mut self, load_addr: Paddr);
}

pub const CPU_SIZE: usize = core::mem::size_of::<Cpu>();

pub fn mycpu() -> &'static mut Cpu {
    unsafe { &mut *(BAO_CPU_BASE as *mut Cpu) }
}

pub fn mem_cpu_boot_alloc_size() -> usize {
    size_of::<Cpu>() + mycpu().addr_space.pt.dscr.lvls * PAGE_SIZE
}

pub struct SyncToken {
    inner: RwLock<SyncTokenInner>,
}

struct SyncTokenInner {
    ready: bool,
    n: usize,
    count: usize,
}

pub static CPU_SYNC_TOKEN: SyncToken = SyncToken {
    inner: RwLock::new(SyncTokenInner {
        ready: false,
        n: 0,
        count: 0,
    }),
};

impl SyncToken {
    pub fn sync_init(&self, n: usize) {
        let mut inner = self.inner.write();
        inner.ready = true;
        inner.n = n;
        inner.count = 0;
    }

    pub fn sync_barrier(&self) {
        while !self.inner.read().ready {}
        let mut inner = self.inner.write();
        inner.count += 1;
        let next_count = align(inner.count, inner.n);
        drop(inner);

        while self.inner.read().count < next_count { }
    }

    pub fn sync_and_clear_msg(&self) {
        while !self.inner.read().ready {}
        let mut inner = self.inner.write();
        inner.count += 1;
        let next_count = align(inner.count, inner.n);
        drop(inner);

        while self.inner.read().count < next_count {
            // todo: handle cpu messages
        }
        self.sync_barrier();
    }
}

pub fn init(cpu_id: CpuID, load_addr: Paddr) {
    let mycpu = mycpu();
    mycpu.id = cpu_id;
    mycpu.handling_msgs = false;
    mycpu.arch_init(load_addr);

    if mycpu.is_master() {
        CPU_SYNC_TOKEN.sync_init(PLATFORM.cpu_num);
        // todo: ipi_cpumsg_handler_num
    }
    CPU_SYNC_TOKEN.sync_barrier();
}
