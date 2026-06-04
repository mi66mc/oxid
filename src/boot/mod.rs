pub mod limine;

use crate::memory::MemoryMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FramebufferInfo {
    pub address: *mut u8,
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub bits_per_pixel: u16,
}

pub struct BootInfo {
    pub framebuffer: Option<FramebufferInfo>,
    pub memory_map: MemoryMap,
    pub bootloader_name: Option<&'static str>,
    pub bootloader_version: Option<&'static str>,
}
