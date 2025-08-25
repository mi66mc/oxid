#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use vga_buffer::buffer::init_writer;
use crate::tests::testable::Testable;

mod progress_bar;
mod vga_buffer;
mod string;
mod tests;

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

impl<T> Testable for T where T: Fn(), {
    fn run(&self) -> () {
        println!("[ {} ] ...", core::any::type_name::<T>());
        self();
        println!("[ {} ] OK!", core::any::type_name::<T>());
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    println!("RUNNING [{}] TESTS", tests.len());
    for (i, test) in tests.iter().enumerate() {
        progress_bar::progress_bar::progress_bar(tests.len(), i + 1, 10);
        test.run();
    }
    println!("[{}] OK TESTS", tests.len());
}


#[cfg(test)]
#[test_case]
fn init_writer_test() {
    init_writer();
}