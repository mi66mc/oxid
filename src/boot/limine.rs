use core::{ptr, slice, str};

use crate::{
    boot::{BootInfo, FramebufferInfo},
    memory::{MemoryMap, MemoryRegion, MemoryRegionKind},
};

const COMMON_MAGIC: [u64; 2] = [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b];
const FRAMEBUFFER_REQUEST_ID: [u64; 4] = [
    COMMON_MAGIC[0],
    COMMON_MAGIC[1],
    0x9d5827dcd881dd75,
    0xa3148604f6fab11b,
];
const MEMMAP_REQUEST_ID: [u64; 4] = [
    COMMON_MAGIC[0],
    COMMON_MAGIC[1],
    0x67cf3d9d378a806f,
    0xe304acdfc50c3c62,
];
const BOOTLOADER_INFO_REQUEST_ID: [u64; 4] = [
    COMMON_MAGIC[0],
    COMMON_MAGIC[1],
    0xf55038d8e2a1202f,
    0x279426fcf5f59740,
];

#[used]
#[unsafe(link_section = ".limine_requests_start")]
static LIMINE_REQUESTS_START: [u64; 4] = [
    0xf6b8f4b39de7d1ae,
    0xfab91a6940fcb9cf,
    0x785c6ed015d3e316,
    0x181e920a7852b9d9,
];

#[used]
#[unsafe(link_section = ".limine_requests")]
static LIMINE_BASE_REVISION: [u64; 3] = [0xf9562b2d5c95a6c8, 0x6a7b384944536bdc, 2];

#[used]
#[unsafe(link_section = ".limine_requests")]
static mut FRAMEBUFFER_REQUEST: Request<FramebufferResponse> = Request::new(FRAMEBUFFER_REQUEST_ID);

#[used]
#[unsafe(link_section = ".limine_requests")]
static mut MEMMAP_REQUEST: Request<MemoryMapResponse> = Request::new(MEMMAP_REQUEST_ID);

#[used]
#[unsafe(link_section = ".limine_requests")]
static mut BOOTLOADER_INFO_REQUEST: Request<BootloaderInfoResponse> =
    Request::new(BOOTLOADER_INFO_REQUEST_ID);

#[used]
#[unsafe(link_section = ".limine_requests_end")]
static LIMINE_REQUESTS_END: [u64; 2] = [0xadc0e0531bb10d03, 0x9572709f31764c62];

#[repr(C)]
struct Request<T> {
    id: [u64; 4],
    revision: u64,
    response: *const T,
}

impl<T> Request<T> {
    const fn new(id: [u64; 4]) -> Self {
        Self {
            id,
            revision: 0,
            response: ptr::null(),
        }
    }
}

unsafe impl<T> Sync for Request<T> {}

#[repr(C)]
struct FramebufferResponse {
    revision: u64,
    framebuffer_count: u64,
    framebuffers: *const *const Framebuffer,
}

#[repr(C)]
struct Framebuffer {
    address: *mut u8,
    width: u64,
    height: u64,
    pitch: u64,
    bpp: u16,
    memory_model: u8,
    red_mask_size: u8,
    red_mask_shift: u8,
    green_mask_size: u8,
    green_mask_shift: u8,
    blue_mask_size: u8,
    blue_mask_shift: u8,
    unused: [u8; 7],
    edid_size: u64,
    edid: *const u8,
    mode_count: u64,
    modes: *const *const u8,
}

#[repr(C)]
struct MemoryMapResponse {
    revision: u64,
    entry_count: u64,
    entries: *const *const LimineMemoryMapEntry,
}

#[repr(C)]
struct LimineMemoryMapEntry {
    base: u64,
    length: u64,
    kind: u64,
}

#[repr(C)]
struct BootloaderInfoResponse {
    revision: u64,
    name: *const u8,
    version: *const u8,
}

const MAX_MEMORY_REGIONS: usize = 128;
static mut MEMORY_REGIONS: [MemoryRegion; MAX_MEMORY_REGIONS] =
    [MemoryRegion::empty(); MAX_MEMORY_REGIONS];

pub fn load_boot_info() -> BootInfo {
    unsafe {
        BootInfo {
            framebuffer: framebuffer_info(),
            memory_map: memory_map(),
            bootloader_name: bootloader_name(),
            bootloader_version: bootloader_version(),
        }
    }
}

unsafe fn framebuffer_info() -> Option<FramebufferInfo> {
    let response = unsafe { FRAMEBUFFER_REQUEST.response.as_ref()? };
    if response.framebuffer_count == 0 || response.framebuffers.is_null() {
        return None;
    }

    let framebuffer = unsafe { (*response.framebuffers).as_ref()? };
    Some(FramebufferInfo {
        address: framebuffer.address,
        width: framebuffer.width as usize,
        height: framebuffer.height as usize,
        pitch: framebuffer.pitch as usize,
        bits_per_pixel: framebuffer.bpp,
    })
}

unsafe fn memory_map() -> MemoryMap {
    let Some(response) = (unsafe { MEMMAP_REQUEST.response.as_ref() }) else {
        return MemoryMap::empty();
    };

    if response.entries.is_null() {
        return MemoryMap::empty();
    }

    let count = (response.entry_count as usize).min(MAX_MEMORY_REGIONS);
    let storage = ptr::addr_of_mut!(MEMORY_REGIONS).cast::<MemoryRegion>();

    for index in 0..count {
        let entry_ptr = unsafe { *response.entries.add(index) };
        let Some(entry) = (unsafe { entry_ptr.as_ref() }) else {
            continue;
        };

        unsafe {
            storage.add(index).write(MemoryRegion {
                base: entry.base,
                length: entry.length,
                kind: MemoryRegionKind::from_limine(entry.kind),
            });
        }
    }

    MemoryMap::new(unsafe { slice::from_raw_parts(storage, count) })
}

unsafe fn bootloader_name() -> Option<&'static str> {
    unsafe { bootloader_string(|response| response.name) }
}

unsafe fn bootloader_version() -> Option<&'static str> {
    unsafe { bootloader_string(|response| response.version) }
}

unsafe fn bootloader_string(field: fn(&BootloaderInfoResponse) -> *const u8) -> Option<&'static str> {
    let response = unsafe { BOOTLOADER_INFO_REQUEST.response.as_ref()? };
    let ptr = field(response);
    if ptr.is_null() {
        return None;
    }

    let mut len = 0;
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }

    str::from_utf8(unsafe { slice::from_raw_parts(ptr, len) }).ok()
}
