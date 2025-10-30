use core::ptr::{addr_of, addr_of_mut};

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct GdtPtr {
    limit: u16,
    base: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Tss {
    reserved0: u32,
    rsp0: u64,
    rsp1: u64,
    rsp2: u64,
    reserved1: u64,
    ist1: u64,
    ist2: u64,
    ist3: u64,
    ist4: u64,
    ist5: u64,
    ist6: u64,
    ist7: u64,
    reserved2: u64,
    reserved3: u16,
    iopb_offset: u16,
}

const GDT_ENTRIES: usize = 7;

static mut GDT: [GdtEntry; GDT_ENTRIES] = [GdtEntry {
    limit_low: 0,
    base_low: 0,
    base_middle: 0,
    access: 0,
    granularity: 0,
    base_high: 0,
}; GDT_ENTRIES];

static mut GDT_POINTER: GdtPtr = GdtPtr {
    limit: 0,
    base: 0,
};

static mut KERNEL_TSS: Tss = Tss {
    reserved0: 0,
    rsp0: 0,
    rsp1: 0,
    rsp2: 0,
    reserved1: 0,
    ist1: 0,
    ist2: 0,
    ist3: 0,
    ist4: 0,
    ist5: 0,
    ist6: 0,
    ist7: 0,
    reserved2: 0,
    reserved3: 0,
    iopb_offset: 0,
};

#[repr(align(16))]
struct KernelStack([u8; 4096]);

static mut KERNEL_STACK: KernelStack = KernelStack([0; 4096]);

unsafe fn gdt_set_entry(
    index: usize,
    base: u32,
    limit: u32,
    access: u8,
    granularity: u8,
) {
    unsafe {
        GDT[index].base_low = (base & 0xFFFF) as u16;
        GDT[index].base_middle = ((base >> 16) & 0xFF) as u8;
        GDT[index].base_high = ((base >> 24) & 0xFF) as u8;
        GDT[index].limit_low = (limit & 0xFFFF) as u16;
        GDT[index].granularity = ((limit >> 16) & 0x0F) as u8;
        GDT[index].granularity |= granularity & 0xF0;
        GDT[index].access = access;
    }
}

unsafe fn gdt_set_tss(
    index: usize,
    base: u64,
    limit: u32,
    access: u8,
    granularity: u8,
) {
    let mut desc_low: u64 = 0;
    desc_low |= (limit & 0xFFFF) as u64;
    desc_low |= ((base & 0xFFFF) as u64) << 16;
    desc_low |= (((base >> 16) & 0xFF) as u64) << 32;
    desc_low |= (access as u64) << 40;
    desc_low |= (((limit >> 16) & 0x0F) as u64) << 48;
    desc_low |= ((granularity & 0xF0) as u64) << 48;
    desc_low |= (((base >> 24) & 0xFF) as u64) << 56;

    let desc_high: u64 = (base >> 32) & 0xFFFFFFFF;

    unsafe {
        let dst = addr_of_mut!(GDT) as *mut u8;
        let dst = dst.add(index * core::mem::size_of::<u64>());
        let vals = [desc_low, desc_high];
        
        for k in 0..2 {
            let mut v = vals[k];
            for i in 0..8 {
                *dst.add(k * 8 + i) = (v & 0xFF) as u8;
                v >>= 8;
            }
        }
    }
}

pub unsafe fn gdt_init() {
    unsafe {
        GDT_POINTER.limit = (core::mem::size_of::<GdtEntry>() * GDT_ENTRIES - 1) as u16;
        GDT_POINTER.base = addr_of!(GDT) as u64;
        
        gdt_set_entry(0, 0, 0, 0, 0);
        gdt_set_entry(1, 0, 0xFFFFF, 0x9A, 0xAF);
        gdt_set_entry(2, 0, 0xFFFFF, 0x92, 0xCF);
        gdt_set_entry(3, 0, 0xFFFFF, 0xFA, 0xAF);
        gdt_set_entry(4, 0, 0xFFFFF, 0xF2, 0xCF);
        
        // Clear TSS
        let tss_ptr = addr_of_mut!(KERNEL_TSS) as *mut u8;
        for i in 0..core::mem::size_of::<Tss>() {
            *tss_ptr.add(i) = 0;
        }
        
        KERNEL_TSS.rsp0 = addr_of!(KERNEL_STACK.0) as u64 + 4096;
        KERNEL_TSS.iopb_offset = core::mem::size_of::<Tss>() as u16;
        
        gdt_set_tss(
            5,
            addr_of!(KERNEL_TSS) as u64,
            (core::mem::size_of::<Tss>() - 1) as u32,
            0x89,
            0x00,
        );

        gdt_load();
    }
}

unsafe fn gdt_load() {
    unsafe {
        core::arch::asm!(
            "lgdt [{}]",
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            "push 0x08",
            "lea rax, [rip + 2f]",
            "push rax",
            "retfq",
            "2:",
            in(reg) addr_of!(GDT_POINTER),
            out("rax") _,
            options(nostack)
        );
        
        core::arch::asm!(
            "ltr {0:x}",
            in(reg) 0x28u16,
        );
    }
}