use aarch64::regs::{ELR_EL2, ESR_EL2, FAR_EL2};
use tock_registers::interfaces::Readable;

use crate::{
    arch::aarch64::{
        gic::{gicc_dir, gicc_eoir, gicc_iar},
        intr::interrupts_handle, psci::{is_psci_smc_call, psci_smc_handler},
    },
    baocore::{
        emul::EmulAccess,
        intr::IntrHandleResult,
        vm::{myvcpu, myvm},
    },
    debug, println,
    util::bit64_extract,
};

use super::sysregs::*;

fn aborts_data_lower(iss: u64, far: u64, il: u64) {
    if iss & ESR_ISS_DA_ISV_BIT == 0 || iss & ESR_ISS_DA_FnV_BIT != 0 {
        panic!("no information to handle data abort ({:#x?})", far);
    }
    let dsfc = bit64_extract(iss, ESR_ISS_DA_DSFC_OFF, ESR_ISS_DA_DSFC_LEN) & (0xf << 2); // Data Fault Status Code

    if dsfc != ESR_ISS_DA_DSFC_TRNSLT && dsfc != ESR_ISS_DA_DSFC_PERMIS {
        panic!("data abort is not translation fault - cant deal with it");
    }

    let addr = far;
    let access = EmulAccess {
        addr,
        width: 1 << bit64_extract(iss, ESR_ISS_DA_SAS_OFF, ESR_ISS_DA_SAS_LEN),
        write: iss & ESR_ISS_DA_WnR_BIT != 0,
        reg: bit64_extract(iss, ESR_ISS_DA_SRT_OFF, ESR_ISS_DA_SRT_LEN),
    };
    println!("Access = {:#x?}", access);

    let handler = myvm().emul_get_mem(addr).unwrap();
    if handler(&access) {
        let pc_step = 2 + 2 * il;
        myvcpu().write_pc(myvcpu().read_pc() + pc_step);
    } else {
        println!("data abort emulation failed: access = {:#x?}", access);
    }
}

pub fn smc_handler() {
    let fid = myvcpu().read_reg(0);
    let arg1 = myvcpu().read_reg(1);
    let arg2 = myvcpu().read_reg(2);
    let arg3 = myvcpu().read_reg(3);
    debug!(
        "smc: fid = {:#x?}, arg1 = {:#x?}, arg2 = {:#x?}, arg3 = {:#x?}",
        fid, arg1, arg2, arg3
    );

    if is_psci_smc_call(fid) {
        // is SMC_HANDLER
        let ret = psci_smc_handler(fid, arg1, arg2, arg3);
        myvcpu().write_reg(0, ret);
    } else {
        panic!("not a psci call {:#x?}", fid);
    }
    myvcpu().write_pc(myvcpu().read_pc() + 4);
    // todo!("fid = {:#x?}", fid);
}

#[no_mangle]
fn sync_exceptions_handler() {
    let esr = ESR_EL2.extract();

    let far = FAR_EL2.get();
    let hpfar = read_reg!(hpfar_el2);
    let iss = esr.read(ESR_EL2::ISS);
    let il = esr.read(ESR_EL2::IL);

    let ipa_fault_addr = (far & 0xfff) | (hpfar << 8);

    match esr.read_as_enum(ESR_EL2::EC) {
        Some(ESR_EL2::EC::Value::Unknown) => {
            panic!("Unknown exception!");
        }
        Some(ESR_EL2::EC::Value::DataAbortLowerEL) => {
            println!(
                "DA: ISS = {:#x}, \
                 GuestVaddr = {:#x?}, \
                 GuestPaddr = {:#x?}, \
                 fault_addr = {:#x?}, \
                 instruction_addr = {:#x?}",
                iss,
                far,
                hpfar << 8,
                ipa_fault_addr,
                ELR_EL2.get(),
            );
            aborts_data_lower(iss, ipa_fault_addr, il);
        }
        Some(ESR_EL2::EC::Value::SMC64) => {
            println!("SMC64: instruction_addr = {:#x?}", ELR_EL2.get());
            smc_handler();
        }
        _ => {
            panic!(
                "Unsupported synchronous exception: ESR = {:#x} (EC {:#08b}, ISS {:#x}) \
                instruction_addr = {:#x?}",
                esr.get(),
                esr.read(ESR_EL2::EC),
                iss,
                ELR_EL2.get(),
            );
        }
    }
}

#[no_mangle]
fn gic_handler() {
    let ack = gicc_iar();
    let id = ack & ((1 << 24) - 1);
    // debug!("gic_handler: id = {}", id);

    if id < 1020 {
        let res = interrupts_handle(id as _);
        gicc_eoir(ack as _);
        if let IntrHandleResult::HandledByHyp = res {
            gicc_dir(ack as _);
        }
    }
}

#[no_mangle]
fn internal_sync_exceptions_handler() {
    println!("internal_sync_exceptions_handler");
    let esr = ESR_EL2.extract();

    let far = FAR_EL2.get();
    let hpfar = read_reg!(hpfar_el2);
    let iss = esr.read(ESR_EL2::ISS);

    let ipa_fault_addr = (far & 0xfff) | (hpfar << 8);

    match esr.read_as_enum(ESR_EL2::EC) {
        Some(ESR_EL2::EC::Value::Unknown) => {
            panic!("Unknown exception!");
        }
        Some(ESR_EL2::EC::Value::InstrAbortCurrentEL)
        | Some(ESR_EL2::EC::Value::DataAbortCurrentEL) => {
            panic!(
                "PageFault: ISS = {:#x}, \
                 GuestVaddr = {:#x?}, \
                 GuestPaddr = {:#x?}, \
                 fault_addr = {:#x?}, \
                 instruction_addr = {:#x?}",
                iss,
                far,
                hpfar << 8,
                ipa_fault_addr,
                ELR_EL2.get(),
            );
        }
        _ => {
            panic!(
                "Unsupported synchronous exception: ESR = {:#x} (EC {:#08b}, ISS {:#x}) \
                instruction_addr = {:#x?}",
                esr.get(),
                esr.read(ESR_EL2::EC),
                iss,
                ELR_EL2.get(),
            );
        }
    }
}
