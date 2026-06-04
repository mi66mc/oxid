use core::{arch::asm, fmt::Write, mem::size_of};

use super::{halt_loop, serial};

const KERNEL_CODE_SELECTOR: u16 = 0x08;
const IDT_ENTRY_COUNT: usize = 256;
const INTERRUPT_GATE: u16 = 0x8e00;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    options: u16,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            options: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    fn new(handler: HandlerAddress) -> Self {
        let address = handler.address();
        Self {
            offset_low: address as u16,
            selector: KERNEL_CODE_SELECTOR,
            options: INTERRUPT_GATE,
            offset_mid: (address >> 16) as u16,
            offset_high: (address >> 32) as u32,
            reserved: 0,
        }
    }
}

#[repr(C, align(16))]
struct InterruptDescriptorTable {
    entries: [IdtEntry; IDT_ENTRY_COUNT],
}

impl InterruptDescriptorTable {
    const fn new() -> Self {
        Self {
            entries: [IdtEntry::missing(); IDT_ENTRY_COUNT],
        }
    }
}

#[repr(C, packed)]
struct DescriptorTablePointer {
    limit: u16,
    base: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InterruptStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

enum HandlerAddress {
    WithoutErrorCode(extern "x86-interrupt" fn(InterruptStackFrame)),
    WithErrorCode(extern "x86-interrupt" fn(InterruptStackFrame, u64)),
}

impl HandlerAddress {
    fn address(&self) -> u64 {
        match self {
            Self::WithoutErrorCode(handler) => *handler as usize as u64,
            Self::WithErrorCode(handler) => *handler as usize as u64,
        }
    }
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn init() {
    unsafe {
        IDT.entries[0] = IdtEntry::new(HandlerAddress::WithoutErrorCode(divide_error_handler));
        IDT.entries[3] = IdtEntry::new(HandlerAddress::WithoutErrorCode(breakpoint_handler));
        IDT.entries[6] = IdtEntry::new(HandlerAddress::WithoutErrorCode(invalid_opcode_handler));
        IDT.entries[8] = IdtEntry::new(HandlerAddress::WithErrorCode(double_fault_handler));
        IDT.entries[13] = IdtEntry::new(HandlerAddress::WithErrorCode(
            general_protection_fault_handler,
        ));
        IDT.entries[14] = IdtEntry::new(HandlerAddress::WithErrorCode(page_fault_handler));

        let pointer = DescriptorTablePointer {
            limit: (size_of::<InterruptDescriptorTable>() - 1) as u16,
            base: (&raw const IDT) as *const InterruptDescriptorTable as u64,
        };
        load_idt(&pointer);
    }
}

#[allow(dead_code)]
pub fn trigger_breakpoint() {
    unsafe {
        asm!("int3", options(nomem, nostack));
    }
}

unsafe fn load_idt(pointer: &DescriptorTablePointer) {
    unsafe {
        asm!("lidt [{}]", in(reg) pointer, options(readonly, nostack, preserves_flags));
    }
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    report_exception("Divide Error", 0, None, &stack_frame);
    halt_loop();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    report_exception("Breakpoint", 3, None, &stack_frame);
    halt_loop();
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    report_exception("Invalid Opcode", 6, None, &stack_frame);
    halt_loop();
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    report_exception("Double Fault", 8, Some(error_code), &stack_frame);
    halt_loop();
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    report_exception("General Protection Fault", 13, Some(error_code), &stack_frame);
    halt_loop();
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    report_exception("Page Fault", 14, Some(error_code), &stack_frame);
    halt_loop();
}

fn report_exception(
    name: &str,
    vector: u8,
    error_code: Option<u64>,
    stack_frame: &InterruptStackFrame,
) {
    let mut port = serial::com1();
    port.init();
    let _ = writeln!(port, "Exception: {name}");
    let _ = writeln!(port, "Vector: {vector}");
    if let Some(error_code) = error_code {
        let _ = writeln!(port, "Error code: {error_code:#x}");
    }
    let _ = writeln!(
        port,
        "Instruction pointer: {:#x}",
        stack_frame.instruction_pointer
    );
    let _ = writeln!(port, "Kernel halted.");
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::{HandlerAddress, IdtEntry, INTERRUPT_GATE, KERNEL_CODE_SELECTOR};

    extern "x86-interrupt" fn test_handler(_stack_frame: super::InterruptStackFrame) {}

    #[test]
    fn idt_entry_uses_kernel_code_selector() {
        let entry = IdtEntry::new(HandlerAddress::WithoutErrorCode(test_handler));

        assert_eq!(entry.selector, KERNEL_CODE_SELECTOR);
    }

    #[test]
    fn idt_entry_uses_interrupt_gate_options() {
        let entry = IdtEntry::new(HandlerAddress::WithoutErrorCode(test_handler));

        assert_eq!(entry.options, INTERRUPT_GATE);
    }
}
