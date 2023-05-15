pub mod gic_defs;
mod gicd;
mod gicv3;
mod vgicv3;
pub mod vgic;

use crate::{
    baocore::{
        cpu::{mycpu, CPU_SYNC_TOKEN},
        intr::interrupts_reserve,
    },
    platform::PLATFORM,
    write_reg,
};

use self::{gicv3::GicV3, vgic::gic_maintenance_handler};

use super::{
    armv8_a::fences::isb,
    sysregs::{ICC_SRE_ENB_BIT, ICC_SRE_SRE_BIT},
};
use gicv3::gic_map_mmio;
use spin::Once;

pub enum GicVersion {
    GicVersion2,
    GicVersion3,
}

pub const GIC_VERSION: GicVersion = GicVersion::GicVersion3;
pub use vgicv3::{vgic_init, gicd_reg_mask, VGIC_ENABLE_MASK};
type Gic = GicV3;

static mut GIC: Once<Gic> = Once::new();

pub fn init() {
    if let GicVersion::GicVersion3 = GIC_VERSION {
        write_reg!(ICC_SRE_EL2, ICC_SRE_SRE_BIT | ICC_SRE_ENB_BIT);
        isb();
    }

    if mycpu().is_master() {
        let (gicd, gicr) = gic_map_mmio();
        let mut gic = Gic::new(gicd, gicr);
        interrupts_reserve(
            PLATFORM.arch.gic.maintenance_id,
            gic_maintenance_handler as _,
        );
        gic.gicd_init();
        unsafe {
            GIC.call_once(|| gic);
        }
    }

    CPU_SYNC_TOKEN.sync_and_clear_msg();

    unsafe {
        GIC.get_mut().unwrap().each_cpu_init(mycpu().id);
    }
}
