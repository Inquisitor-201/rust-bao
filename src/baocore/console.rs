use core::{
    fmt::{self, Write},
    sync::atomic::{AtomicU64, Ordering},
};

use spin::Mutex;

use crate::{
    arch::aarch64::armv8_a::fences::fence_sync_write,
    baocore::mmu::sections::SEC_HYP_GLOBAL,
    platform::{drivers::Uart, PLATFORM},
    util::num_pages,
};

use super::cpu::{mycpu, CPU_SYNC_TOKEN};

static UART: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    if mycpu().is_master() {
        let uart_va = mycpu()
            .addr_space
            .mem_alloc_map_dev(
                SEC_HYP_GLOBAL,
                PLATFORM.console_base,
                None,
                num_pages(core::mem::size_of::<Uart>()),
            )
            .unwrap();

        fence_sync_write();
        let uart = unsafe { &mut *(uart_va as *mut Uart) };
        uart.init();
        uart.enable();
        UART.store(uart_va, Ordering::Release);
    }
    CPU_SYNC_TOKEN.sync_and_clear_msg();
}

fn console_write(s: &str) {
    let uart = UART.load(Ordering::Acquire);
    if uart == 0 {
        return;
    }
    let uart = unsafe { &mut *(uart as *mut Uart) };
    uart.puts(s);
}

static PRINT_LOCK: Mutex<()> = Mutex::new(());

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        console_write(s);
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    let _lock = PRINT_LOCK.lock();
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::baocore::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    () => { print!("\n") };
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::baocore::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
