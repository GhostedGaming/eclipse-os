use crate::mem::mem;
use core::ptr::null_mut;

const PAGE_SIZE: usize = 4096;
const ENTRY_COUNT: usize = 512;

// Page table entry flags
const PRESENT: u64 = 1 << 0;
const WRITABLE: u64 = 1 << 1;
const USER: u64 = 1 << 2;
const WRITE_THROUGH: u64 = 1 << 3;
const CACHE_DISABLE: u64 = 1 << 4;
const ACCESSED: u64 = 1 << 5;
const DIRTY: u64 = 1 << 6;
const HUGE_PAGE: u64 = 1 << 7;
const GLOBAL: u64 = 1 << 8;
const NO_EXECUTE: u64 = 1 << 63;

#[repr(C, align(4096))]
pub struct PageTable {
    entries: [u64; ENTRY_COUNT],
}

impl PageTable {
    const fn new() -> Self {
        Self {
            entries: [0; ENTRY_COUNT],
        }
    }

    fn zero(&mut self) {
        for i in 0..ENTRY_COUNT {
            self.entries[i] = 0;
        }
    }
}

static mut KERNEL_PML4: *mut PageTable = null_mut();

/// Page mapping flags
#[derive(Clone, Copy)]
pub struct PageFlags {
    pub writable: bool,
    pub user: bool,
    pub write_through: bool,
    pub cache_disable: bool,
    pub no_execute: bool,
}

impl PageFlags {
    pub const fn kernel() -> Self {
        Self {
            writable: true,
            user: false,
            write_through: false,
            cache_disable: false,
            no_execute: false,
        }
    }

    pub const fn kernel_ro() -> Self {
        Self {
            writable: false,
            user: false,
            write_through: false,
            cache_disable: false,
            no_execute: false,
        }
    }

    pub const fn user() -> Self {
        Self {
            writable: true,
            user: true,
            write_through: false,
            cache_disable: false,
            no_execute: false,
        }
    }

    fn to_entry_flags(&self) -> u64 {
        let mut flags = PRESENT;
        if self.writable {
            flags |= WRITABLE;
        }
        if self.user {
            flags |= USER;
        }
        if self.write_through {
            flags |= WRITE_THROUGH;
        }
        if self.cache_disable {
            flags |= CACHE_DISABLE;
        }
        if self.no_execute {
            flags |= NO_EXECUTE;
        }
        flags
    }
}

/// Initialize the virtual memory manager
pub unsafe fn vmm_init() -> Result<(), &'static str> {
    unsafe {
        // Allocate PML4 (top-level page table)
        let pml4_frame = mem::alloc_frame().ok_or("Failed to allocate PML4")?;
        KERNEL_PML4 = pml4_frame as *mut PageTable;
        (*KERNEL_PML4).zero();

        Ok(())
    }
}

/// Get the current PML4 address
pub unsafe fn get_kernel_pml4() -> u64 {
    unsafe { KERNEL_PML4 as u64 }
}

/// Extract page table indices from virtual address
fn get_page_table_indices(vaddr: u64) -> (usize, usize, usize, usize) {
    let pml4_idx = ((vaddr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((vaddr >> 30) & 0x1FF) as usize;
    let pd_idx = ((vaddr >> 21) & 0x1FF) as usize;
    let pt_idx = ((vaddr >> 12) & 0x1FF) as usize;
    (pml4_idx, pdpt_idx, pd_idx, pt_idx)
}

/// Get or create a page table entry
unsafe fn get_or_create_table(entry: &mut u64, flags: u64) -> Result<*mut PageTable, &'static str> {
    unsafe {
        if *entry & PRESENT == 0 {
            // Allocate new page table
            let frame = mem::alloc_frame().ok_or("Out of memory")?;
            let table = frame as *mut PageTable;
            (*table).zero();
            *entry = frame | flags;
        }

        let addr = *entry & 0x000F_FFFF_FFFF_F000;
        Ok(addr as *mut PageTable)
    }
}

/// Map a virtual address to a physical address
pub unsafe fn map_page(
    pml4: *mut PageTable,
    vaddr: u64,
    paddr: u64,
    flags: PageFlags,
) -> Result<(), &'static str> {
    unsafe {
        let (pml4_idx, pdpt_idx, pd_idx, pt_idx) = get_page_table_indices(vaddr);
        let entry_flags = flags.to_entry_flags();

        // Navigate/create page tables
        let pdpt = get_or_create_table(&mut (*pml4).entries[pml4_idx], entry_flags)?;
        let pd = get_or_create_table(&mut (*pdpt).entries[pdpt_idx], entry_flags)?;
        let pt = get_or_create_table(&mut (*pd).entries[pd_idx], entry_flags)?;

        // Map the page
        if (*pt).entries[pt_idx] & PRESENT != 0 {
            return Err("Page already mapped");
        }

        (*pt).entries[pt_idx] = (paddr & 0x000F_FFFF_FFFF_F000) | entry_flags;

        // Invalidate TLB entry
        core::arch::asm!("invlpg [{}]", in(reg) vaddr, options(nostack));

        Ok(())
    }
}

/// Unmap a virtual address
pub unsafe fn unmap_page(pml4: *mut PageTable, vaddr: u64) -> Result<u64, &'static str> {
    unsafe {
        let (pml4_idx, pdpt_idx, pd_idx, pt_idx) = get_page_table_indices(vaddr);

        // Navigate page tables
        if (*pml4).entries[pml4_idx] & PRESENT == 0 {
            return Err("Page not mapped");
        }
        let pdpt = ((*pml4).entries[pml4_idx] & 0x000F_FFFF_FFFF_F000) as *mut PageTable;

        if (*pdpt).entries[pdpt_idx] & PRESENT == 0 {
            return Err("Page not mapped");
        }
        let pd = ((*pdpt).entries[pdpt_idx] & 0x000F_FFFF_FFFF_F000) as *mut PageTable;

        if (*pd).entries[pd_idx] & PRESENT == 0 {
            return Err("Page not mapped");
        }
        let pt = ((*pd).entries[pd_idx] & 0x000F_FFFF_FFFF_F000) as *mut PageTable;

        if (*pt).entries[pt_idx] & PRESENT == 0 {
            return Err("Page not mapped");
        }

        let paddr = (*pt).entries[pt_idx] & 0x000F_FFFF_FFFF_F000;
        (*pt).entries[pt_idx] = 0;

        // Invalidate TLB entry
        core::arch::asm!("invlpg [{}]", in(reg) vaddr, options(nostack));

        Ok(paddr)
    }
}

/// Translate virtual address to physical address
pub unsafe fn virt_to_phys(pml4: *mut PageTable, vaddr: u64) -> Option<u64> {
    unsafe {
        let (pml4_idx, pdpt_idx, pd_idx, pt_idx) = get_page_table_indices(vaddr);
        let offset = vaddr & 0xFFF;

        // Navigate page tables
        if (*pml4).entries[pml4_idx] & PRESENT == 0 {
            return None;
        }
        let pdpt = ((*pml4).entries[pml4_idx] & 0x000F_FFFF_FFFF_F000) as *mut PageTable;

        if (*pdpt).entries[pdpt_idx] & PRESENT == 0 {
            return None;
        }
        let pd = ((*pdpt).entries[pdpt_idx] & 0x000F_FFFF_FFFF_F000) as *mut PageTable;

        if (*pd).entries[pd_idx] & PRESENT == 0 {
            return None;
        }
        let pt = ((*pd).entries[pd_idx] & 0x000F_FFFF_FFFF_F000) as *mut PageTable;

        if (*pt).entries[pt_idx] & PRESENT == 0 {
            return None;
        }

        let paddr = (*pt).entries[pt_idx] & 0x000F_FFFF_FFFF_F000;
        Some(paddr | offset)
    }
}

/// Map a range of memory
pub unsafe fn map_range(
    pml4: *mut PageTable,
    vaddr: u64,
    paddr: u64,
    size: usize,
    flags: PageFlags,
) -> Result<(), &'static str> {
    let page_count = (size + PAGE_SIZE - 1) / PAGE_SIZE;

    unsafe {
        for i in 0..page_count {
            let v = vaddr + (i * PAGE_SIZE) as u64;
            let p = paddr + (i * PAGE_SIZE) as u64;
            map_page(pml4, v, p, flags)?;
        }
    }

    Ok(())
}

/// Identity map a range (virtual address = physical address)
pub unsafe fn identity_map_range(
    pml4: *mut PageTable,
    paddr: u64,
    size: usize,
    flags: PageFlags,
) -> Result<(), &'static str> {
    unsafe { map_range(pml4, paddr, paddr, size, flags) }
}

/// Load a page table (set CR3)
pub unsafe fn load_page_table(pml4_addr: u64) {
    unsafe {
        core::arch::asm!("mov cr3, {}", in(reg) pml4_addr, options(nostack));
    }
}

/// Get current page table address (read CR3)
pub unsafe fn get_current_page_table() -> u64 {
    let cr3: u64;
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack));
    }
    cr3 & 0x000F_FFFF_FFFF_F000
}