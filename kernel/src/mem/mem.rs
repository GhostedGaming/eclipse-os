//! A FIFO (First In, First Out) allocator implemented using a linked list.
//! Each block in the allocator is represented as a node in the list.

use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::null_mut,
    mem,
};
use limine::{memory_map::EntryType, response::MemoryMapResponse};

static mut HEAP_START: *mut u8 = null_mut();
static mut HEAP_OFFSET: usize = 0;

/// Represents a block in the linked list allocator.
struct LinkedListBlock {
    size: usize,
    next: *mut LinkedListBlock,
    prev: *mut LinkedListBlock,
}

/// A linked list structure used for managing blocks.
struct LinkedList {
    head: *mut LinkedListBlock,
    tail: *mut LinkedListBlock,
    count: usize,
}

impl LinkedList {
    const fn new() -> Self {
        LinkedList {
            head: null_mut(),
            tail: null_mut(),
            count: 0,
        }
    }

    unsafe fn push_back(&mut self, block: *mut LinkedListBlock) {
        (*block).next = null_mut();
        (*block).prev = self.tail;

        if !self.tail.is_null() {
            (*self.tail).next = block;
        } else {
            self.head = block;
        }

        self.tail = block;
        self.count += 1;
    }

    unsafe fn pop_front(&mut self) -> *mut LinkedListBlock {
        if self.head.is_null() {
            return null_mut();
        }

        let front = self.head;
        self.head = (*front).next;

        if !self.head.is_null() {
            (*self.head).prev = null_mut();
        } else {
            self.tail = null_mut();
        }

        self.count -= 1;
        front
    }

    unsafe fn remove(&mut self, block: *mut LinkedListBlock) {
        if (*block).prev.is_null() {
            self.head = (*block).next;
        } else {
            (*(*block).prev).next = (*block).next;
        }

        if (*block).next.is_null() {
            self.tail = (*block).prev;
        } else {
            (*(*block).next).prev = (*block).prev;
        }

        self.count -= 1;
    }

    fn is_empty(&self) -> bool {
        self.head.is_null()
    }
}

static mut FREE_LIST: LinkedList = LinkedList {
    head: null_mut(),
    tail: null_mut(),
    count: 0,
};

pub struct LinkAllocator;

unsafe impl GlobalAlloc for LinkAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut current = FREE_LIST.head;
        
        while !current.is_null() {
            if (*current).size >= layout.size() {
                FREE_LIST.remove(current);
                
                return (current as *mut u8).add(mem::size_of::<LinkedListBlock>());
            }
            current = (*current).next;
        }

        if HEAP_START.is_null() {
            return null_mut();
        }

        let total_size = mem::size_of::<LinkedListBlock>() + layout.size();
        
        let align = layout.align().max(mem::align_of::<LinkedListBlock>());
        let offset = (HEAP_OFFSET + align - 1) & !(align - 1);
        
        let block = HEAP_START.add(offset) as *mut LinkedListBlock;
        (*block).size = layout.size();
        (*block).next = null_mut();
        (*block).prev = null_mut();
        
        HEAP_OFFSET = offset + total_size;
        
        (block as *mut u8).add(mem::size_of::<LinkedListBlock>())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() {
            return;
        }

        let block = ptr.sub(mem::size_of::<LinkedListBlock>()) as *mut LinkedListBlock;
        
        FREE_LIST.push_back(block);
    }
}

#[global_allocator]
static ALLOCATOR: LinkAllocator = LinkAllocator;

/// Initialize the allocator
pub unsafe fn init_allocator(memory_map: &MemoryMapResponse) {
    FREE_LIST = LinkedList::new();
    
    for entry in memory_map.entries() {
        if entry.entry_type == EntryType::USABLE && entry.length > 1024 * 1024 {
            HEAP_START = (entry.base + 0xFFFF800000000000) as *mut u8;
            HEAP_OFFSET = 0;
            break;
        }
    }
    
    if HEAP_START.is_null() {
        panic!("No usable memory found in memory map");
    }
}

const PAGE_SIZE: usize = 4096;
const ENTRIES_PER_TABLE: usize = 512;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PhysAddr(u64);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct VirtAddr(u64);

impl PhysAddr {
    pub fn new(addr: u64) -> Self {
        PhysAddr(addr & 0x000F_FFFF_FFFF_F000)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl VirtAddr {
    pub fn new(addr: u64) -> Self {
        VirtAddr(addr)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
    
    pub fn page_offset(&self) -> usize {
        (self.0 & 0xFFF) as usize
    }
    
    pub fn p4_index(&self) -> usize {
        ((self.0 >> 39) & 0x1FF) as usize
    }
    
    pub fn p3_index(&self) -> usize {
        ((self.0 >> 30) & 0x1FF) as usize
    }
    
    pub fn p2_index(&self) -> usize {
        ((self.0 >> 21) & 0x1FF) as usize
    }
    
    pub fn p1_index(&self) -> usize {
        ((self.0 >> 12) & 0x1FF) as usize
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub const PRESENT: u64 = 1 << 0;
    pub const WRITABLE: u64 = 1 << 1;
    pub const USER: u64 = 1 << 2;
    pub const WRITE_THROUGH: u64 = 1 << 3;
    pub const NO_CACHE: u64 = 1 << 4;
    pub const ACCESSED: u64 = 1 << 5;
    pub const DIRTY: u64 = 1 << 6;
    pub const HUGE: u64 = 1 << 7;
    pub const GLOBAL: u64 = 1 << 8;
    pub const NO_EXECUTE: u64 = 1 << 63;
    
    pub fn new() -> Self {
        PageTableEntry(0)
    }
    
    pub fn is_present(&self) -> bool {
        (self.0 & Self::PRESENT) != 0
    }
    
    pub fn set_addr(&mut self, addr: PhysAddr, flags: u64) {
        self.0 = addr.as_u64() | flags;
    }
    
    pub fn get_addr(&self) -> PhysAddr {
        PhysAddr::new(self.0 & 0x000F_FFFF_FFFF_F000)
    }
    
    pub fn clear(&mut self) {
        self.0 = 0;
    }
}

#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; ENTRIES_PER_TABLE],
}

impl PageTable {
    pub fn new() -> Self {
        PageTable {
            entries: [PageTableEntry::new(); ENTRIES_PER_TABLE],
        }
    }
    
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.clear();
        }
    }
}

static mut FRAME_BITMAP: *mut u64 = null_mut();
static mut TOTAL_FRAMES: usize = 0;
static mut BITMAP_SIZE: usize = 0;

pub struct FrameAllocator;

impl FrameAllocator {
    pub unsafe fn init(memory_map: &MemoryMapResponse) {
        let mut max_addr = 0u64;
        
        for entry in memory_map.entries() {
            let end = entry.base + entry.length;
            if end > max_addr {
                max_addr = end;
            }
        }
        
        TOTAL_FRAMES = (max_addr / PAGE_SIZE as u64) as usize;
        BITMAP_SIZE = (TOTAL_FRAMES + 63) / 64;
        
        for entry in memory_map.entries() {
            if entry.entry_type == EntryType::USABLE && entry.length >= (BITMAP_SIZE * 8) as u64 {
                FRAME_BITMAP = (entry.base + 0xFFFF800000000000) as *mut u64;
                
                for i in 0..BITMAP_SIZE {
                    *FRAME_BITMAP.add(i) = 0xFFFFFFFFFFFFFFFF;
                }
                
                let bitmap_frames = (BITMAP_SIZE * 8 + PAGE_SIZE - 1) / PAGE_SIZE;
                for i in 0..bitmap_frames {
                    let frame = (entry.base as usize / PAGE_SIZE) + i;
                    Self::mark_used(frame);
                }
                
                break;
            }
        }
        
        for entry in memory_map.entries() {
            if entry.entry_type == EntryType::USABLE {
                let start_frame = (entry.base as usize) / PAGE_SIZE;
                let frame_count = (entry.length as usize) / PAGE_SIZE;
                
                for i in 0..frame_count {
                    Self::mark_free(start_frame + i);
                }
            }
        }
    }
    
    unsafe fn mark_free(frame: usize) {
        if frame >= TOTAL_FRAMES {
            return;
        }
        let index = frame / 64;
        let bit = frame % 64;
        *FRAME_BITMAP.add(index) |= 1u64 << bit;
    }
    
    unsafe fn mark_used(frame: usize) {
        if frame >= TOTAL_FRAMES {
            return;
        }
        let index = frame / 64;
        let bit = frame % 64;
        *FRAME_BITMAP.add(index) &= !(1u64 << bit);
    }
    
    pub unsafe fn alloc_frame() -> Option<PhysAddr> {
        for i in 0..BITMAP_SIZE {
            let bitmap = *FRAME_BITMAP.add(i);
            if bitmap != 0 {
                let bit = bitmap.trailing_zeros() as usize;
                let frame = i * 64 + bit;
                
                if frame < TOTAL_FRAMES {
                    Self::mark_used(frame);
                    return Some(PhysAddr::new((frame * PAGE_SIZE) as u64));
                }
            }
        }
        None
    }
    
    pub unsafe fn free_frame(addr: PhysAddr) {
        let frame = (addr.as_u64() / PAGE_SIZE as u64) as usize;
        Self::mark_free(frame);
    }
}

static mut KERNEL_PAGE_TABLE: *mut PageTable = null_mut();

pub struct VMM;

impl VMM {
    pub unsafe fn init(memory_map: &MemoryMapResponse) {
        FrameAllocator::init(memory_map);
        
        let mut cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
        KERNEL_PAGE_TABLE = (cr3 & 0x000F_FFFF_FFFF_F000) as *mut PageTable;
    }
    
    unsafe fn get_or_create_table(entry: &mut PageTableEntry) -> Option<*mut PageTable> {
        if entry.is_present() {
            Some(entry.get_addr().as_u64() as *mut PageTable)
        } else {
            let frame = FrameAllocator::alloc_frame()?;
            let table = frame.as_u64() as *mut PageTable;
            (*table).zero();
            entry.set_addr(frame, PageTableEntry::PRESENT | PageTableEntry::WRITABLE | PageTableEntry::USER);
            Some(table)
        }
    }
    
    pub unsafe fn map_page(virt: VirtAddr, phys: PhysAddr, flags: u64) -> Option<()> {
        const HHDM_OFFSET: u64 = 0xFFFF800000000000;

        let p4 = &mut *(((KERNEL_PAGE_TABLE as u64) | HHDM_OFFSET) as *mut PageTable);

        let p3_entry = &mut p4.entries[virt.p4_index()];
        let p3 = if p3_entry.is_present() {
            ((p3_entry.get_addr().as_u64()) | HHDM_OFFSET) as *mut PageTable
        } else {
            let frame = FrameAllocator::alloc_frame()?;
            let table = ((frame.as_u64()) | HHDM_OFFSET) as *mut PageTable;
            (*table).zero();
            p3_entry.set_addr(frame, PageTableEntry::PRESENT | PageTableEntry::WRITABLE | PageTableEntry::USER);
            table
        };

        let p2_entry = &mut (*p3).entries[virt.p3_index()];
        let p2 = if p2_entry.is_present() {
            ((p2_entry.get_addr().as_u64()) | HHDM_OFFSET) as *mut PageTable
        } else {
            let frame = FrameAllocator::alloc_frame()?;
            let table = ((frame.as_u64()) | HHDM_OFFSET) as *mut PageTable;
            (*table).zero();
            p2_entry.set_addr(frame, PageTableEntry::PRESENT | PageTableEntry::WRITABLE | PageTableEntry::USER);
            table
        };

        let p1_entry = &mut (*p2).entries[virt.p2_index()];
        let p1 = if p1_entry.is_present() {
            ((p1_entry.get_addr().as_u64()) | HHDM_OFFSET) as *mut PageTable
        } else {
            let frame = FrameAllocator::alloc_frame()?;
            let table = ((frame.as_u64()) | HHDM_OFFSET) as *mut PageTable;
            (*table).zero();
            p1_entry.set_addr(frame, PageTableEntry::PRESENT | PageTableEntry::WRITABLE | PageTableEntry::USER);
            table
        };

        (*p1).entries[virt.p1_index()].set_addr(phys, flags | PageTableEntry::PRESENT);

        core::arch::asm!("invlpg [{}]", in(reg) virt.as_u64(), options(nostack, preserves_flags));

        Some(())
    }
    
    pub unsafe fn unmap_page(virt: VirtAddr) {
        let p4 = &mut *KERNEL_PAGE_TABLE;
        
        if !p4.entries[virt.p4_index()].is_present() {
            return;
        }
        
        let p3 = p4.entries[virt.p4_index()].get_addr().as_u64() as *mut PageTable;
        if !(*p3).entries[virt.p3_index()].is_present() {
            return;
        }
        
        let p2 = (*p3).entries[virt.p3_index()].get_addr().as_u64() as *mut PageTable;
        if !(*p2).entries[virt.p2_index()].is_present() {
            return;
        }
        
        let p1 = (*p2).entries[virt.p2_index()].get_addr().as_u64() as *mut PageTable;
        (*p1).entries[virt.p1_index()].clear();
        
        core::arch::asm!("invlpg [{}]", in(reg) virt.as_u64(), options(nostack, preserves_flags));
    }
    
    pub unsafe fn enable_paging() {
        core::arch::asm!(
            "mov cr3, {}",
            in(reg) KERNEL_PAGE_TABLE as u64,
            options(nostack, preserves_flags)
        );
    }
    
    pub fn get_page_table() -> *mut PageTable {
        unsafe { KERNEL_PAGE_TABLE }
    }
}