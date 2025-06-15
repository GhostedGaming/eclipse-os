use core::mem;
use spin::Mutex;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

#[repr(C, packed)]
struct GdtPointer {
    limit: u16,
    base: u64,
}

#[repr(C, packed)]
pub struct TaskStateSegment {
    reserved1: u32,
    pub rsp: [u64; 3],
    reserved2: u64,
    pub ist: [u64; 7],
    reserved3: u64,
    reserved4: u16,
    pub iomap_base: u16,
}

impl TaskStateSegment {
    pub const fn new() -> TaskStateSegment {
        TaskStateSegment {
            reserved1: 0,
            rsp: [0; 3],
            reserved2: 0,
            ist: [0; 7],
            reserved3: 0,
            reserved4: 0,
            iomap_base: 0,
        }
    }
}

static STACK: Mutex<[u8; 4096 * 5]> = Mutex::new([0; 4096 * 5]);
static TSS: Mutex<TaskStateSegment> = Mutex::new(TaskStateSegment::new());
static GDT: Mutex<[u64; 8]> = Mutex::new([0; 8]);

// Create 64-bit GDT entries directly as u64
fn create_code_segment() -> u64 {
    // 64-bit kernel code segment
    // Base: 0, Limit: 0xFFFFF, Access: 0x9A (present, ring 0, code, readable)
    // Flags: 0x20 (64-bit, granularity 4KB)
    0x00AF9A000000FFFF
}

fn create_data_segment() -> u64 {
    // 64-bit kernel data segment  
    // Base: 0, Limit: 0xFFFFF, Access: 0x92 (present, ring 0, data, writable)
    // Flags: 0x00 (32-bit for data segments in 64-bit mode)
    0x00CF92000000FFFF
}

fn create_tss_entry(tss_addr: u64) -> (u64, u64) {
    let base = tss_addr;
    let limit = (mem::size_of::<TaskStateSegment>() - 1) as u64;
    
    // Low 64 bits of TSS descriptor
    let low = (limit & 0xFFFF) |
              ((base & 0xFFFF) << 16) |
              (((base >> 16) & 0xFF) << 32) |
              (0x89u64 << 40) | // Present, TSS Available
              (((limit >> 16) & 0xF) << 48) |
              (((base >> 24) & 0xFF) << 56);
    
    // High 64 bits (upper 32 bits of base address)
    let high = base >> 32;
    
    (low, high)
}

pub fn init() {
    // Set up the double fault stack
    let stack_end = {
        let stack = STACK.lock();
        stack.as_ptr() as u64 + stack.len() as u64
    };
    
    // Set up TSS
    {
        let mut tss = TSS.lock();
        tss.ist[DOUBLE_FAULT_IST_INDEX as usize] = stack_end;
    }
    
    // Get TSS address for creating the TSS entry
    let tss_addr = {
        let tss = TSS.lock();
        &*tss as *const TaskStateSegment as u64
    };

    // Create GDT entries
    let gdt_addr = {
        let mut gdt = GDT.lock();
        gdt[0] = 0; // Null descriptor
        gdt[1] = create_code_segment(); // Kernel code segment (selector 0x08)
        gdt[2] = create_data_segment(); // Kernel data segment (selector 0x10)
        
        // TSS entry (takes 2 slots) - selectors 0x18 and 0x20
        let (tss_low, tss_high) = create_tss_entry(tss_addr);
        gdt[3] = tss_low;
        gdt[4] = tss_high;
        
        gdt.as_ptr() as u64
    };

    unsafe {
        // Load GDT
        let gdt_ptr = GdtPointer {
            limit: (mem::size_of::<[u64; 8]>() - 1) as u16,
            base: gdt_addr,
        };

        core::arch::asm!(
            "lgdt [{}]",
            in(reg) &gdt_ptr,
            options(readonly, nostack, preserves_flags)
        );

        // Load code segment
        core::arch::asm!(
            "push {sel}",
            "lea {tmp}, [2f + rip]",
            "push {tmp}",
            "retfq",
            "2:",
            sel = in(reg) 0x08u64, // Code segment selector
            tmp = lateout(reg) _,
            options(preserves_flags)
        );

        // Load data segment selectors
        core::arch::asm!(
            "mov {0}, {1}",
            "mov ds, {0:x}",
            "mov es, {0:x}",
            "mov fs, {0:x}",
            "mov gs, {0:x}",
            "mov ss, {0:x}",
            out(reg) _,
            in(reg) 0x10u16, // Data segment selector
            options(nostack, preserves_flags)
        );

        // Load TSS
        core::arch::asm!(
            "ltr {0:x}",
            in(reg) 0x18u16, // TSS segment selector
            options(nostack, preserves_flags)
        );
    }
}