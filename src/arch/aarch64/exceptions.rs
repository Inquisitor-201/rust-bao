use aarch64::regs::{ELR_EL2, ESR_EL2, FAR_EL2};
use tock_registers::interfaces::Readable;

use crate::{baocore::{vm::{myvm, myvcpu}, emul::EmulAccess}, println, util::bit64_extract};

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
        reg: bit64_extract(iss, ESR_ISS_DA_SRT_OFF, ESR_ISS_DA_SRT_LEN)
    };
    println!("Access = {:#x?}", access);

    let handler = myvm().emul_get_mem(addr).unwrap();
    if handler(&access) {
        let pc_step = 2 + 2 * il;
        myvcpu().write_pc(myvcpu().read_pc() +  pc_step);
    } else {
        println!("data abort emulation failed: access = {:#x?}", access);
    }
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
            aborts_data_lower(iss, far, il);
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
    println!("gic_handler");
    loop {}
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
