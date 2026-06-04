#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]
#![cfg_attr(target_os = "none", feature(abi_x86_interrupt))]
#![cfg_attr(target_os = "none", feature(custom_test_frameworks))]
#![cfg_attr(target_os = "none", test_runner(crate::kernel_test_runner))]
#![cfg_attr(target_os = "none", reexport_test_harness_main = "kernel_test_main")]

#[cfg(target_os = "none")]
use core::panic::PanicInfo;

#[cfg(target_os = "none")]
mod arch;
#[cfg(target_os = "none")]
mod boot;
#[cfg(target_os = "none")]
mod drivers;
#[cfg(target_os = "none")]
mod kernel;
#[cfg(any(test, target_os = "none"))]
mod memory;
#[cfg(all(test, not(target_os = "none")))]
mod text;

#[cfg(not(target_os = "none"))]
fn main() {}

#[cfg(target_os = "none")]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    {
        use core::fmt::Write;

        let mut early_serial = arch::x86_64::serial::com1();
        early_serial.init();
        let _ = early_serial.write_str("Oxid entry\n");
    }

    arch::x86_64::init();

    let boot_info = boot::limine::load_boot_info();

    #[cfg(test)]
    kernel_test_main();

    kernel::init(&boot_info)
}

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::panic(info)
}

#[cfg(all(test, target_os = "none"))]
pub fn kernel_test_runner(_tests: &[&dyn Fn()]) {
    arch::x86_64::halt_loop()
}
