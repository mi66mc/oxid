use core::{arch::asm, mem::size_of};

const KERNEL_CODE_SELECTOR: u16 = 0x08;

const fn descriptor(access: u8, flags: u8) -> u64 {
    let limit = 0x000f_ffffu64;
    (limit & 0xffff)
        | (((limit >> 16) & 0x0f) << 48)
        | ((access as u64) << 40)
        | ((flags as u64) << 52)
}

#[repr(C, align(8))]
struct GlobalDescriptorTable {
    entries: [u64; 3],
}

#[repr(C, packed)]
struct DescriptorTablePointer {
    limit: u16,
    base: u64,
}

static GDT: GlobalDescriptorTable = GlobalDescriptorTable {
    entries: [
        0,
        descriptor(0b1001_1010, 0b1010),
        descriptor(0b1001_0010, 0b1100),
    ],
};

pub fn init() {
    let pointer = DescriptorTablePointer {
        limit: (size_of::<GlobalDescriptorTable>() - 1) as u16,
        base: (&GDT as *const GlobalDescriptorTable) as u64,
    };

    unsafe {
        load_gdt(&pointer);
        reload_segments();
    }
}

unsafe fn load_gdt(pointer: &DescriptorTablePointer) {
    unsafe {
        asm!("lgdt [{}]", in(reg) pointer, options(readonly, nostack, preserves_flags));
    }
}

unsafe fn reload_segments() {
    unsafe {
        asm!(
            "push {code_selector}",
            "lea {target}, [rip + 2f]",
            "push {target}",
            "retfq",
            "2:",
            code_selector = in(reg) u64::from(KERNEL_CODE_SELECTOR),
            target = lateout(reg) _,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::descriptor;

    #[test]
    fn encodes_kernel_code_descriptor_bits() {
        assert_eq!(descriptor(0b1001_1010, 0b1010), 0x00af_9a00_0000_ffff);
    }

    #[test]
    fn encodes_kernel_data_descriptor_bits() {
        assert_eq!(descriptor(0b1001_0010, 0b1100), 0x00cf_9200_0000_ffff);
    }
}
