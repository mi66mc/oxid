use core::panic::PanicInfo;

use crate::{arch::x86_64, boot::BootInfo, drivers::console, kprintln};

pub fn init(boot_info: &BootInfo) -> ! {
    console::init(boot_info.framebuffer);

    kprintln!("Oxid kernel initialized");

    if let Some(name) = boot_info.bootloader_name {
        match boot_info.bootloader_version {
            Some(version) => kprintln!("Bootloader: {} {}", name, version),
            None => kprintln!("Bootloader: {}", name),
        }
    }

    if let Some(framebuffer) = boot_info.framebuffer {
        kprintln!(
            "Framebuffer: {}x{}x{}",
            framebuffer.width,
            framebuffer.height,
            framebuffer.bits_per_pixel
        );
    } else {
        kprintln!("Framebuffer: unavailable");
    }

    kprintln!("Memory regions: {}", boot_info.memory_map.entries().len());
    kprintln!(
        "Usable memory: {} KiB",
        boot_info.memory_map.total_usable_bytes() / 1024
    );
    kprintln!("Kernel halted.");

    x86_64::halt_loop()
}

pub fn panic(info: &PanicInfo) -> ! {
    kprintln!("Kernel panic: {}", info);
    x86_64::halt_loop()
}
