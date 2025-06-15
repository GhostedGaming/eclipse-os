#![no_std]
#![no_main]

use core::panic::PanicInfo;
use eclipse_os::{QemuExitCode, exit_qemu, serial_print, serial_println};
use spin::Mutex;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");
    eclipse_os::gdt::init();
    init_test_idt();

    // trigger a stack overflow
    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
    volatile::Volatile::new(0).read(); // prevent tail recursion optimizations
}

// IDT Entry structure
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    const fn new() -> Self {
        IdtEntry {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    fn set_handler(&mut self, handler: u64, ist_index: u8) {
        self.offset_low = handler as u16;
        self.offset_mid = (handler >> 16) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.selector = 0x08; // Kernel code segment
        self.ist = ist_index;
        self.type_attr = 0x8E; // Present, DPL=0, Interrupt Gate
    }
}

#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

#[repr(C)]
pub struct InterruptStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

static TEST_IDT: Mutex<[IdtEntry; 256]> = Mutex::new([IdtEntry::new(); 256]);

// Assembly wrapper for double fault handler
core::arch::global_asm!(
    r#"
.section .text

.macro SAVE_REGS
    push rax
    push rcx
    push rdx
    push rbx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
.endm

.macro RESTORE_REGS
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rbp
    pop rbx
    pop rdx
    pop rcx
    pop rax
.endm

.global test_double_fault_wrapper
test_double_fault_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 128  // Point to interrupt stack frame (after error code)
    mov rsi, [rsp + 120]  // Error code
    call test_double_fault_handler_rust
    // This should never return
    cli
    hlt
"#
);

unsafe extern "C" {
    fn test_double_fault_wrapper();
}

pub fn init_test_idt() {
    let idt_addr = {
        let mut idt = TEST_IDT.lock();
        
        // Set double fault handler (interrupt 8)
        idt[8].set_handler(
            test_double_fault_wrapper as u64, 
            eclipse_os::gdt::DOUBLE_FAULT_IST_INDEX as u8
        );

        idt.as_ptr() as u64
    };

    unsafe {
        let idt_ptr = IdtPointer {
            limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: idt_addr,
        };
        core::arch::asm!(
            "lidt [{}]",
            in(reg) &idt_ptr,
            options(readonly, nostack, preserves_flags)
        );
    }
}

#[unsafe(no_mangle)]
extern "C" fn test_double_fault_handler_rust(
    _stack_frame: *const InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    eclipse_os::test_panic_handler(info)
}
