pub const SMC32_STDSRVC_FID_VALUE: u64 = 0x84000000;
pub const SMC64_STDSRVC_FID_VALUE: u64 = 0xc4000000;

pub const fn is_psci_smc_call(fid: u64) -> bool {
    let f = fid & 0xff000000;
    f == SMC32_STDSRVC_FID_VALUE || f == SMC64_STDSRVC_FID_VALUE
}

const PSCI_VERSION: u64 = 0x84000000;
const PSCI_MIG_INFO_TYPE: u64 = 0x84000006;
const PSCI_CPU_ON_SMC64: u64 = 0xc4000003;

const PSCI_VERSION_0_2: u64 = 2;
const PSCI_TOS_NOT_PRESENT_MP: u64 = 2;

pub fn psci_smc_handler(fid: u64, _arg1: u64, _arg2: u64, _arg3: u64) -> u64 {
    match fid {
        PSCI_VERSION => PSCI_VERSION_0_2,       // PSCI_VERSION_0_2
        PSCI_MIG_INFO_TYPE => PSCI_TOS_NOT_PRESENT_MP, // PSCI_TOS_NOT_PRESENT_MP
        PSCI_CPU_ON_SMC64 => 1,
        _ => {
            panic!("unknown psci call");
        }
    }
}
