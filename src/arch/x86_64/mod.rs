pub mod gdt;
pub mod idt;
pub mod port;
pub mod serial;

pub fn init() {
    gdt::init();
    idt::init();
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn halt() {
    unsafe {
        core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
pub fn halt() {}

pub fn halt_loop() -> ! {
    loop {
        halt();
    }
}
