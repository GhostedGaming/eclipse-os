use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use crate::mem::mem;
use crate::mem::vmm::{self, PageFlags};

const PAGE_SIZE: usize = 4096;
const MIN_BLOCK_SIZE: usize = 16;
const HEAP_START: u64 = 0xFFFF_8000_0000_0000; // Kernel heap virtual address
const INITIAL_HEAP_SIZE: usize = 4 * 1024 * 1024; // Start with 4MB

#[repr(align(16))]
struct HeapBlock {
    size: usize,
    next: *mut HeapBlock,
    free: bool,
}

struct HeapAllocator {
    heap_start: AtomicUsize,
    heap_end: AtomicUsize,
    initialized: AtomicBool,
}

impl HeapAllocator {
    const fn new() -> Self {
        Self {
            heap_start: AtomicUsize::new(0),
            heap_end: AtomicUsize::new(0),
            initialized: AtomicBool::new(false),
        }
    }

    unsafe fn init(&self, pml4: *mut u8) -> Result<(), &'static str> {
        if self.initialized.load(Ordering::SeqCst) {
            return Err("Heap already initialized");
        }

        let page_count = INITIAL_HEAP_SIZE / PAGE_SIZE;
        let heap_start = HEAP_START as usize;
        let heap_end = heap_start + INITIAL_HEAP_SIZE;

        // Allocate and map pages for the heap
        unsafe {
            for i in 0..page_count {
                let vaddr = (heap_start + i * PAGE_SIZE) as u64;
                let paddr = mem::alloc_frame().ok_or("Out of physical memory")?;
                
                vmm::map_page(
                    pml4 as *mut vmm::PageTable,
                    vaddr,
                    paddr,
                    PageFlags::kernel(),
                )?;
            }
        }

        // Initialize the first free block
        let initial_block = heap_start as *mut HeapBlock;
        unsafe {
            (*initial_block).size = INITIAL_HEAP_SIZE - core::mem::size_of::<HeapBlock>();
            (*initial_block).next = null_mut();
            (*initial_block).free = true;
        }

        self.heap_start.store(heap_start, Ordering::SeqCst);
        self.heap_end.store(heap_end, Ordering::SeqCst);
        self.initialized.store(true, Ordering::SeqCst);

        Ok(())
    }

    unsafe fn expand_heap(&self, pml4: *mut u8, additional_pages: usize) -> Result<(), &'static str> {
        let current_end = self.heap_end.load(Ordering::SeqCst);
        
        unsafe {
            for i in 0..additional_pages {
                let vaddr = (current_end + i * PAGE_SIZE) as u64;
                let paddr = mem::alloc_frame().ok_or("Out of physical memory")?;
                
                vmm::map_page(
                    pml4 as *mut vmm::PageTable,
                    vaddr,
                    paddr,
                    PageFlags::kernel(),
                )?;
            }
        }

        let new_end = current_end + (additional_pages * PAGE_SIZE);
        self.heap_end.store(new_end, Ordering::SeqCst);

        // Add the new space to the free list
        let new_block = current_end as *mut HeapBlock;
        unsafe {
            (*new_block).size = (additional_pages * PAGE_SIZE) - core::mem::size_of::<HeapBlock>();
            (*new_block).next = null_mut();
            (*new_block).free = true;
        }

        // Merge with existing free blocks
        self.merge_free_blocks();

        Ok(())
    }

    unsafe fn find_free_block(&self, size: usize, align: usize) -> Option<*mut HeapBlock> {
        let heap_start = self.heap_start.load(Ordering::SeqCst);
        let heap_end = self.heap_end.load(Ordering::SeqCst);
        let mut current = heap_start as *mut HeapBlock;

        unsafe {
            while !current.is_null() && (current as usize) < heap_end {
                if (*current).free && (*current).size >= size {
                    // Check alignment
                    let data_ptr = (current as *mut u8).add(core::mem::size_of::<HeapBlock>());
                    let aligned_ptr = align_up(data_ptr as usize, align) as *mut u8;
                    let alignment_offset = aligned_ptr as usize - data_ptr as usize;
                    
                    if (*current).size >= size + alignment_offset {
                        return Some(current);
                    }
                }
                current = (*current).next;
            }
        }

        None
    }

    unsafe fn split_block(&self, block: *mut HeapBlock, required_size: usize) {
        unsafe {
            let block_size = (*block).size;
            let remaining_size = block_size - required_size - core::mem::size_of::<HeapBlock>();

            if remaining_size >= MIN_BLOCK_SIZE {
                let new_block = ((block as *mut u8)
                    .add(core::mem::size_of::<HeapBlock>())
                    .add(required_size)) as *mut HeapBlock;

                (*new_block).size = remaining_size;
                (*new_block).next = (*block).next;
                (*new_block).free = true;

                (*block).size = required_size;
                (*block).next = new_block;
            }
        }
    }

    unsafe fn merge_free_blocks(&self) {
        let heap_start = self.heap_start.load(Ordering::SeqCst);
        let heap_end = self.heap_end.load(Ordering::SeqCst);
        let mut current = heap_start as *mut HeapBlock;

        unsafe {
            while !current.is_null() && (current as usize) < heap_end {
                if (*current).free {
                    let mut next = (*current).next;
                    
                    while !next.is_null() && (*next).free {
                        // Check if blocks are adjacent
                        let current_end = (current as usize) + 
                            core::mem::size_of::<HeapBlock>() + 
                            (*current).size;
                        
                        if current_end == next as usize {
                            // Merge current and next
                            (*current).size += core::mem::size_of::<HeapBlock>() + (*next).size;
                            (*current).next = (*next).next;
                            next = (*current).next;
                        } else {
                            break;
                        }
                    }
                }
                current = (*current).next;
            }
        }
    }
}

unsafe impl Sync for HeapAllocator {}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !self.initialized.load(Ordering::SeqCst) {
            return null_mut();
        }

        let size = layout.size().max(MIN_BLOCK_SIZE);
        let align = layout.align();

        unsafe {
            if let Some(block) = self.find_free_block(size, align) {
                self.split_block(block, size);
                (*block).free = false;

                let data_ptr = (block as *mut u8).add(core::mem::size_of::<HeapBlock>());
                let aligned_ptr = align_up(data_ptr as usize, align) as *mut u8;
                
                aligned_ptr
            } else {
                // Try to expand heap if we're out of memory
                // Note: This is simplified - in production you'd get PML4 from somewhere
                null_mut()
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() || !self.initialized.load(Ordering::SeqCst) {
            return;
        }

        let heap_start = self.heap_start.load(Ordering::SeqCst);
        let heap_end = self.heap_end.load(Ordering::SeqCst);

        unsafe {
            // Find the block header
            let block = (ptr as *mut u8)
                .sub(core::mem::size_of::<HeapBlock>()) as *mut HeapBlock;

            if (block as usize) >= heap_start && (block as usize) < heap_end {
                (*block).free = true;
                self.merge_free_blocks();
            }
        }
    }
}

#[global_allocator]
static ALLOCATOR: HeapAllocator = HeapAllocator::new();

/// Initialize the kernel heap with PMM/VMM
pub unsafe fn init_heap(pml4: *mut u8) -> Result<(), &'static str> {
    unsafe {
        ALLOCATOR.init(pml4)
    }
}

/// Expand the heap by a number of pages
pub unsafe fn expand_heap(pml4: *mut u8, pages: usize) -> Result<(), &'static str> {
    unsafe {
        ALLOCATOR.expand_heap(pml4, pages)
    }
}

// Required for alloc error handling
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}