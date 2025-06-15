use crate::{gdt, hlt_loop};
use alloc::format;
use spin::Mutex;
use crate::uefi_text_buffer::print_message;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// PIC constants
const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;
const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;

#[repr(C)]
pub struct InterruptStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

impl core::fmt::Debug for InterruptStackFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("InterruptStackFrame")
            .field("instruction_pointer", &format_args!("{:#x}", self.instruction_pointer))
            .field("code_segment", &format_args!("{:#x}", self.code_segment))
            .field("cpu_flags", &format_args!("{:#x}", self.cpu_flags))
            .field("stack_pointer", &format_args!("{:#x}", self.stack_pointer))
            .field("stack_segment", &format_args!("{:#x}", self.stack_segment))
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PageFaultErrorCode {
    bits: u64,
}

impl PageFaultErrorCode {
    pub fn protection_violation(&self) -> bool {
        self.bits & 1 != 0
    }
    pub fn caused_by_write(&self) -> bool {
        self.bits & 2 != 0
    }
    pub fn user_mode(&self) -> bool {
        self.bits & 4 != 0
    }
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

static IDT: Mutex<[IdtEntry; 256]> = Mutex::new([IdtEntry::new(); 256]);

pub struct ChainedPics {
    pics: [Pic; 2],
}

struct Pic {
    offset: u8,
    command: u16,
    data: u16,
}

impl ChainedPics {
    pub const unsafe fn new(offset1: u8, offset2: u8) -> ChainedPics {
        ChainedPics {
            pics: [
                Pic {
                    offset: offset1,
                    command: PIC1_COMMAND,
                    data: PIC1_DATA,
                },
                Pic {
                    offset: offset2,
                    command: PIC2_COMMAND,
                    data: PIC2_DATA,
                },
            ],
        }
    }

    pub unsafe fn initialize(&mut self) {
        // Save masks
        let mask1 = unsafe { inb(PIC1_DATA) };
        let mask2 = unsafe { inb(PIC2_DATA) };

        // Start initialization
        unsafe {
            outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
            outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);

            // Set offsets
            outb(PIC1_DATA, self.pics[0].offset);
            outb(PIC2_DATA, self.pics[1].offset);

            // Configure chaining
            outb(PIC1_DATA, 4); // PIC2 at IRQ2
            outb(PIC2_DATA, 2); // Cascade identity

            // Set mode
            outb(PIC1_DATA, ICW4_8086);
            outb(PIC2_DATA, ICW4_8086);

            // Restore masks
            outb(PIC1_DATA, mask1);
            outb(PIC2_DATA, mask2);
        }
    }

    pub unsafe fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
        if interrupt_id >= self.pics[1].offset {
            unsafe { outb(PIC2_COMMAND, 0x20) };
        }
        unsafe { outb(PIC1_COMMAND, 0x20) };
    }
}

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// Port I/O functions
unsafe fn outb(port: u16, value: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        core::arch::asm!(
            "in al, dx",
            out("al") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}

// Read CR2 register
unsafe fn read_cr2() -> u64 {
    let value: u64;
    unsafe {
        core::arch::asm!(
            "mov {}, cr2",
            out(reg) value,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}

// Assembly interrupt handler wrappers
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

.global divide_error_wrapper
divide_error_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 120  // Point to interrupt stack frame
    call divide_error_handler_rust
    RESTORE_REGS
    iretq

.global debug_wrapper
debug_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 120
    call debug_handler_rust
    RESTORE_REGS
    iretq

.global nmi_wrapper
nmi_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 120
    call nmi_handler_rust
    RESTORE_REGS
    iretq

.global breakpoint_wrapper
breakpoint_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 120
    call breakpoint_handler_rust
    RESTORE_REGS
    iretq

.global overflow_wrapper
overflow_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 120
    call overflow_handler_rust
    RESTORE_REGS
    iretq

.global bound_range_exceeded_wrapper
bound_range_exceeded_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 120
    call bound_range_exceeded_handler_rust
    RESTORE_REGS
    iretq

.global invalid_opcode_wrapper
invalid_opcode_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 120
    call invalid_opcode_handler_rust
    RESTORE_REGS
    iretq

.global device_not_available_wrapper
device_not_available_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 120
    call device_not_available_handler_rust
    RESTORE_REGS
    iretq

.global double_fault_wrapper
double_fault_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 120
    mov rsi, [rsp + 120]  // Error code
    call double_fault_handler_rust
    // This should never return
    cli
    hlt

.global invalid_tss_wrapper
invalid_tss_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 128  // Point to interrupt stack frame (after error code)
    mov rsi, [rsp + 120]  // Error code
    call invalid_tss_handler_rust
    RESTORE_REGS
    add rsp, 8  // Remove error code
    iretq

.global segment_not_present_wrapper
segment_not_present_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 128
    mov rsi, [rsp + 120]
    call segment_not_present_handler_rust
    RESTORE_REGS
    add rsp, 8
    iretq

.global stack_segment_fault_wrapper
stack_segment_fault_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 128
    mov rsi, [rsp + 120]
    call stack_segment_fault_handler_rust
    RESTORE_REGS
    add rsp, 8
    iretq

.global general_protection_fault_wrapper
general_protection_fault_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 128
    mov rsi, [rsp + 120]
    call general_protection_fault_handler_rust
    RESTORE_REGS
    add rsp, 8
    iretq

.global page_fault_wrapper
page_fault_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 128
    mov rsi, [rsp + 120]
    call page_fault_handler_rust
    RESTORE_REGS
    add rsp, 8
    iretq

.global x87_floating_point_wrapper
x87_floating_point_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 120
    call x87_floating_point_handler_rust
    RESTORE_REGS
    iretq

.global alignment_check_wrapper
alignment_check_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 128
    mov rsi, [rsp + 120]
    call alignment_check_handler_rust
    RESTORE_REGS
    add rsp, 8
    iretq

.global machine_check_wrapper
machine_check_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 120
    call machine_check_handler_rust
    // This should never return
    cli
    hlt

.global simd_floating_point_wrapper
simd_floating_point_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 120
    call simd_floating_point_handler_rust
    RESTORE_REGS
    iretq

.global virtualization_wrapper
virtualization_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 120
    call virtualization_handler_rust
    RESTORE_REGS
    iretq

.global timer_interrupt_wrapper
timer_interrupt_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 120
    call timer_interrupt_handler_rust
    RESTORE_REGS
    iretq

.global keyboard_interrupt_wrapper
keyboard_interrupt_wrapper:
    SAVE_REGS
    mov rdi, rsp
    add rdi, 120
    call keyboard_interrupt_handler_rust
    RESTORE_REGS
    iretq
"#
);

// External assembly functions
unsafe extern "C" {
    fn divide_error_wrapper();
    fn debug_wrapper();
    fn nmi_wrapper();
    fn breakpoint_wrapper();
    fn overflow_wrapper();
    fn bound_range_exceeded_wrapper();
    fn invalid_opcode_wrapper();
    fn device_not_available_wrapper();
    fn double_fault_wrapper();
    fn invalid_tss_wrapper();
    fn segment_not_present_wrapper();
    fn stack_segment_fault_wrapper();
    fn general_protection_fault_wrapper();
    fn page_fault_wrapper();
    fn x87_floating_point_wrapper();
    fn alignment_check_wrapper();
    fn machine_check_wrapper();
    fn simd_floating_point_wrapper();
    fn virtualization_wrapper();
    fn timer_interrupt_wrapper();
    fn keyboard_interrupt_wrapper();
}

pub fn init_idt() {
    let idt_addr = {
        let mut idt = IDT.lock();

        // Exception handlers (0-31)
        idt[0].set_handler(divide_error_wrapper as u64, 0);
        idt[1].set_handler(debug_wrapper as u64, 0);
        idt[2].set_handler(nmi_wrapper as u64, 0);
        idt[3].set_handler(breakpoint_wrapper as u64, 0);
        idt[4].set_handler(overflow_wrapper as u64, 0);
        idt[5].set_handler(bound_range_exceeded_wrapper as u64, 0);
        idt[6].set_handler(invalid_opcode_wrapper as u64, 0);
        idt[7].set_handler(device_not_available_wrapper as u64, 0);
        idt[8].set_handler(double_fault_wrapper as u64, gdt::DOUBLE_FAULT_IST_INDEX as u8);
        // 9 is reserved
        idt[10].set_handler(invalid_tss_wrapper as u64, 0);
        idt[11].set_handler(segment_not_present_wrapper as u64, 0);
        idt[12].set_handler(stack_segment_fault_wrapper as u64, 0);
        idt[13].set_handler(general_protection_fault_wrapper as u64, 0);
        idt[14].set_handler(page_fault_wrapper as u64, 0);
        // 15 is reserved
        idt[16].set_handler(x87_floating_point_wrapper as u64, 0);
        idt[17].set_handler(alignment_check_wrapper as u64, 0);
        idt[18].set_handler(machine_check_wrapper as u64, 0);
        idt[19].set_handler(simd_floating_point_wrapper as u64, 0);
        idt[20].set_handler(virtualization_wrapper as u64, 0);

        // Hardware interrupts
        idt[InterruptIndex::Timer.as_usize()].set_handler(timer_interrupt_wrapper as u64, 0);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler(keyboard_interrupt_wrapper as u64, 0);

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

// Rust interrupt handlers (called from assembly wrappers)
#[unsafe(no_mangle)]
extern "C" fn breakpoint_handler_rust(stack_frame: *const InterruptStackFrame) {
    let stack_frame = unsafe { &*stack_frame };
    print_message(&format!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame));
}

#[unsafe(no_mangle)]
extern "C" fn page_fault_handler_rust(stack_frame: *const InterruptStackFrame, error_code: u64) {
    let stack_frame = unsafe { &*stack_frame };
    let error = PageFaultErrorCode { bits: error_code };
    print_message("EXCEPTION: PAGE FAULT");
    print_message(&format!("Accessed Address: {:#x}", unsafe { read_cr2() }));
    print_message(&format!("Error Code: {:?}", error));
    print_message(&format!("{:#?}", stack_frame));
    hlt_loop();
}

#[unsafe(no_mangle)]
extern "C" fn double_fault_handler_rust(stack_frame: *const InterruptStackFrame, _error_code: u64) -> ! {
    let stack_frame = unsafe { &*stack_frame };
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

#[unsafe(no_mangle)]
extern "C" fn timer_interrupt_handler_rust(_stack_frame: *const InterruptStackFrame) {
    if let Some(_cpu_freq) = crate::time::get_cpu_frequency_hz() {}

    // Call time::tick() to update the system time
    crate::time::tick();

    // Handle all sound timing (beeps, melodies, sequences)
    crate::pc_speaker::timer_tick();

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

#[unsafe(no_mangle)]
extern "C" fn keyboard_interrupt_handler_rust(_stack_frame: *const InterruptStackFrame) {
    let scancode: u8 = unsafe { inb(0x60) };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

#[unsafe(no_mangle)]
extern "C" fn divide_error_handler_rust(stack_frame: *const InterruptStackFrame) {
    let stack_frame = unsafe { &*stack_frame };
    print_message(&format!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame));
    hlt_loop();
}

#[unsafe(no_mangle)]
extern "C" fn debug_handler_rust(stack_frame: *const InterruptStackFrame) {
    let stack_frame = unsafe { &*stack_frame };
    print_message(&format!("EXCEPTION: DEBUG\n{:#?}", stack_frame));
}

#[unsafe(no_mangle)]
extern "C" fn nmi_handler_rust(stack_frame: *const InterruptStackFrame) {
    let stack_frame = unsafe { &*stack_frame };
    print_message(&format!("EXCEPTION: NON-MASKABLE INTERRUPT\n{:#?}", stack_frame));
}

#[unsafe(no_mangle)]
extern "C" fn overflow_handler_rust(stack_frame: *const InterruptStackFrame) {
    let stack_frame = unsafe { &*stack_frame };
    print_message(&format!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame));
    hlt_loop();
}

#[unsafe(no_mangle)]
extern "C" fn bound_range_exceeded_handler_rust(stack_frame: *const InterruptStackFrame) {
    let stack_frame = unsafe { &*stack_frame };
    print_message(&format!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame));
    hlt_loop();
}

#[unsafe(no_mangle)]
extern "C" fn invalid_opcode_handler_rust(stack_frame: *const InterruptStackFrame) {
    let stack_frame = unsafe { &*stack_frame };
    print_message(&format!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame));
    hlt_loop();
}

#[unsafe(no_mangle)]
extern "C" fn device_not_available_handler_rust(stack_frame: *const InterruptStackFrame) {
    let stack_frame = unsafe { &*stack_frame };
    print_message(&format!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame));
    hlt_loop();
}

#[unsafe(no_mangle)]
extern "C" fn invalid_tss_handler_rust(stack_frame: *const InterruptStackFrame, error_code: u64) {
    let stack_frame = unsafe { &*stack_frame };
    print_message("EXCEPTION: INVALID TSS");
    print_message(&format!("Error Code: {:#x}", error_code));
    print_message(&format!("{:#?}", stack_frame));
    hlt_loop();
}

#[unsafe(no_mangle)]
extern "C" fn segment_not_present_handler_rust(stack_frame: *const InterruptStackFrame, error_code: u64) {
    let stack_frame = unsafe { &*stack_frame };
    print_message("EXCEPTION: SEGMENT NOT PRESENT");
    print_message(&format!("Error Code: {:#x}", error_code));
    print_message(&format!("{:#?}", stack_frame));
    hlt_loop();
}

#[unsafe(no_mangle)]
extern "C" fn stack_segment_fault_handler_rust(stack_frame: *const InterruptStackFrame, error_code: u64) {
    let stack_frame = unsafe { &*stack_frame };
    print_message("EXCEPTION: STACK SEGMENT FAULT");
    print_message(&format!("Error Code: {:#x}", error_code));
    print_message(&format!("{:#?}", stack_frame));
    hlt_loop();
}

#[unsafe(no_mangle)]
extern "C" fn general_protection_fault_handler_rust(stack_frame: *const InterruptStackFrame, error_code: u64) {
    let stack_frame = unsafe { &*stack_frame };
    print_message("EXCEPTION: GENERAL PROTECTION FAULT");
    print_message(&format!("Error Code: {:#x}", error_code));
    print_message(&format!("{:#?}", stack_frame));
    hlt_loop();
}

#[unsafe(no_mangle)]
extern "C" fn x87_floating_point_handler_rust(stack_frame: *const InterruptStackFrame) {
    let stack_frame = unsafe { &*stack_frame };
    print_message(&format!("EXCEPTION: x87 FLOATING POINT\n{:#?}", stack_frame));
    hlt_loop();
}

#[unsafe(no_mangle)]
extern "C" fn alignment_check_handler_rust(stack_frame: *const InterruptStackFrame, error_code: u64) {
    let stack_frame = unsafe { &*stack_frame };
    print_message("EXCEPTION: ALIGNMENT CHECK");
    print_message(&format!("Error Code: {:#x}", error_code));
    print_message(&format!("{:#?}", stack_frame));
    hlt_loop();
}

#[unsafe(no_mangle)]
extern "C" fn machine_check_handler_rust(stack_frame: *const InterruptStackFrame) -> ! {
    let stack_frame = unsafe { &*stack_frame };
    panic!("EXCEPTION: MACHINE CHECK\n{:#?}", stack_frame);
}

#[unsafe(no_mangle)]
extern "C" fn simd_floating_point_handler_rust(stack_frame: *const InterruptStackFrame) {
    let stack_frame = unsafe { &*stack_frame };
    print_message(&format!("EXCEPTION: SIMD FLOATING POINT\n{:#?}", stack_frame));
    hlt_loop();
}

#[unsafe(no_mangle)]
extern "C" fn virtualization_handler_rust(stack_frame: *const InterruptStackFrame) {
    let stack_frame = unsafe { &*stack_frame };
    print_message(&format!("EXCEPTION: VIRTUALIZATION\n{:#?}", stack_frame));
    hlt_loop();
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    unsafe {
        core::arch::asm!("int3", options(nomem, nostack));
    }
}