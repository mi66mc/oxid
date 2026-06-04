pub mod port;
pub mod serial;

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
