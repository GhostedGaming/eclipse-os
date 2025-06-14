use core::mem;
use spin::Mutex;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

#[repr(C, packed)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

#[repr(C, packed)]
struct TssEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
    base_upper: u32,
    reserved: u32,
}

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

impl GdtEntry {
    const fn new(base: u32, limit: u32, access: u8, flags: u8) -> Self {
        GdtEntry {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            access,
            granularity: (flags & 0xF0) | (((limit >> 16) & 0x0F) as u8),
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }

    const fn kernel_code() -> Self {
        Self::new(0, 0xFFFFF, 0x9A, 0xA0)
    }

    const fn kernel_data() -> Self {
        Self::new(0, 0xFFFFF, 0x92, 0xC0)
    }
}

fn create_tss_entry(tss_addr: u64) -> (u64, u64) {
    let base = tss_addr;
    let limit = (mem::size_of::<TaskStateSegment>() - 1) as u64;
    
    let low = ((base & 0xFFFF) << 16) |
              (limit & 0xFFFF) |
              (((base >> 16) & 0xFF) << 32) |
              (0x89u64 << 40) |
              (((limit >> 16) & 0xF) << 48) |
              (((base >> 24) & 0xFF) << 56);
    
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
        gdt[1] = unsafe { mem::transmute::<GdtEntry, u64>(GdtEntry::kernel_code()) };
        gdt[2] = unsafe { mem::transmute::<GdtEntry, u64>(GdtEntry::kernel_data()) };
        
        // TSS entry (takes 2 slots)
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

        // Load code segment - changed label from "1:" to "2:"
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

        // Load TSS
        core::arch::asm!(
            "ltr {0:x}",
            in(reg) 0x18u16, // TSS segment selector
            options(nostack, preserves_flags)
        );
    }
}