use spin::{Lazy, RwLock};

use crate::{
    arch::aarch64::{
        defs::PAGE_SIZE,
        intr::{interrupts_arch_init, MAX_INTERUPTS},
    },
    util::bitmap::{BMSpace, Bitmap},
};

use super::types::{IrqID, Vaddr};

static HYP_BM_SPACE: BMSpace = BMSpace([0; PAGE_SIZE]);
static GLOBAL_BM_SPACE: BMSpace = BMSpace([0; PAGE_SIZE]);

static HYP_INTR_BITMAP: Lazy<RwLock<Bitmap>> =
    Lazy::new(|| RwLock::new(Bitmap::new(HYP_BM_SPACE.base(), MAX_INTERUPTS / 8)));
static GLOBAL_INTR_BITMAP: Lazy<RwLock<Bitmap>> =
    Lazy::new(|| RwLock::new(Bitmap::new(GLOBAL_BM_SPACE.base(), MAX_INTERUPTS / 8)));

pub enum IntrHandleResult {
    ForwardToVM,
    HandledByHyp
}

pub fn interrupts_reserve(int_id: IrqID, _handler: Vaddr) {
    if int_id as usize >= MAX_INTERUPTS {
        return;
    }
    HYP_INTR_BITMAP.write().set(int_id as _);
    GLOBAL_INTR_BITMAP.write().set(int_id as _);
}

pub fn interrupts_is_reserved(int_id: IrqID) -> bool {
    HYP_INTR_BITMAP.read().get(int_id as _)
}

pub fn init() {
    interrupts_arch_init();
}
