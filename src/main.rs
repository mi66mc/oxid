#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use core::panic::PanicInfo;
use vga_buffer::buffer::init_writer;

mod vga_buffer;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    init_writer();

    print!("Oxid Kernel");

    loop {}
}