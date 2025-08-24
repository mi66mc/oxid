#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use vga_buffer::buffer::init_writer;

use crate::progress_bar::progress_bar::progress_bar;
use crate::string::string::String;

mod progress_bar;
mod vga_buffer;
mod string;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    init_writer();
    println!("[ OXID KERNEL ]");

    #[cfg(test)]
    test_main();

    println!("> ");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("[ KERNEL PANIC ]");
    println!("{}", info);
    loop {}
}

macro_rules! named_test {
    ($func:ident) => {
        stringify!($func)
    };
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("RUNNING [{}] TESTS", tests.len());
    for (i, test) in tests.iter().enumerate() {
        progress_bar(tests.len(), i + 1, 10);
        test();
    }
    println!("\n[{}] OK TESTS", tests.len());
}


#[cfg(test)]
#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

#[cfg(test)]
#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

#[cfg(test)]
#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}