use crate::{cpu::gdt, hlt_loop, println};
use spin::Mutex;

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
        let mask1 = inb(PIC1_DATA);
        let mask2 = inb(PIC2_DATA);
        
        // Start initialization
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
    
    pub unsafe fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
        if interrupt_id >= self.pics[1].offset {
            outb(PIC2_COMMAND, 0x20);
        }
        outb(PIC1_COMMAND, 0x20);
    }
}

pub static PICS: Mutex<ChainedPics> = 
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// Port I/O functions
unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    value
}

// Read CR2 register
unsafe fn read_cr2() -> u64 {
    let value: u64;
    core::arch::asm!(
        "mov {}, cr2",
        out(reg) value,
        options(nomem, nostack, preserves_flags)
    );
    value
}

pub fn init_idt() {
    let idt_addr = {
        let mut idt = IDT.lock();
        idt[3].set_handler(breakpoint_handler as u64, 0);
        idt[8].set_handler(double_fault_handler as u64, gdt::DOUBLE_FAULT_IST_INDEX as u8);
        idt[14].set_handler(page_fault_handler as u64, 0);
        idt[InterruptIndex::Timer.as_usize()].set_handler(timer_interrupt_handler as u64, 0);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler(keyboard_interrupt_handler as u64, 0);
        
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

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: u64,
) {
    let error = PageFaultErrorCode { bits: error_code };
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:#x}", unsafe { read_cr2() });
    println!("Error Code: {:?}", error);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    if let Some(_cpu_freq) = crate::time::get_cpu_frequency_hz() {}
    
    // Call time::tick() to update the system time
    crate::time::tick();
    
    // Handle all sound timing (beeps, melodies, sequences)
    crate::pc_speaker::timer_tick();
    
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    let scancode: u8 = unsafe { inb(0x60) };
    crate::task::keyboard::add_scancode(scancode);
    
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    unsafe {
        core::arch::asm!("int3", options(nomem, nostack));
    }
}
