use aarch64::regs::{ESR_EL2, FAR_EL2};
use tock_registers::interfaces::Readable;

use crate::println;

#[no_mangle]
fn sync_exceptions_handler() {
    let esr = ESR_EL2.extract();

    let far = FAR_EL2.get();
    let hpfar = read_reg!(hpfar_el2);
    let iss = esr.read(ESR_EL2::ISS);

    let ipa_fault_addr = (far & 0xfff) | (hpfar << 8);

    match esr.read_as_enum(ESR_EL2::EC) {
        Some(ESR_EL2::EC::Value::Unknown) => {
            panic!("Unknown exception!");
        }
        Some(ESR_EL2::EC::Value::InstrAbortLowerEL) => {
            panic!(
                "InstrAbortLowerEL: ISS = {:#x}, \
                 GuestVaddr = {:#x?}, \
                 GuestPaddr = {:#x?}, \
                 fault_addr = {:#x?}",
                iss,
                far,
                hpfar << 8,
                ipa_fault_addr
            );
        }
        _ => {
            panic!(
                "Unsupported synchronous exception: ESR = {:#x} (EC {:#08b}, ISS {:#x})",
                esr.get(),
                esr.read(ESR_EL2::EC),
                iss,
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
    loop {}
}
