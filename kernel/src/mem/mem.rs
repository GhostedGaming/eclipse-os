use core::ptr::{addr_of_mut, null_mut};
use limine::memory_map::EntryType;
use limine::response::MemoryMapResponse;

const PAGE_SIZE: usize = 4096;

#[repr(C)]
struct FreeListBlock {
    size: usize,
    next: *mut FreeListBlock,
}

static mut FREE_LIST_HEAD: *mut FreeListBlock = null_mut();
static mut TOTAL_MEMORY: usize = 0;
static mut FREE_MEMORY: usize = 0;

pub unsafe fn pmm_init(memmap_response: &MemoryMapResponse) {
    unsafe {
        FREE_LIST_HEAD = null_mut();
        TOTAL_MEMORY = 0;
        FREE_MEMORY = 0;

        // Iterate through memory map entries
        for entry in memmap_response.entries() {
            if entry.entry_type == EntryType::USABLE {
                let base = entry.base as usize;
                let length = entry.length as usize;
                
                // Align base address up to page boundary
                let aligned_base = (base + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
                
                // Calculate aligned length
                let offset = aligned_base - base;
                if length <= offset {
                    continue;
                }
                let aligned_length = (length - offset) & !(PAGE_SIZE - 1);
                
                if aligned_length >= PAGE_SIZE {
                    add_free_region(aligned_base, aligned_length);
                    TOTAL_MEMORY += aligned_length;
                    FREE_MEMORY += aligned_length;
                }
            }
        }
        
        // Sort the free list by address for better coalescing
        sort_free_list();
        
        // Merge adjacent blocks
        coalesce_free_list();
    }
}

unsafe fn add_free_region(base: usize, size: usize) {
    // Create a new free block at the base address
    let block = base as *mut FreeListBlock;
    
    unsafe {
        (*block).size = size;
        (*block).next = FREE_LIST_HEAD;
        FREE_LIST_HEAD = block;
    }
}

/// Sort the free list by address using insertion sort
unsafe fn sort_free_list() {
    unsafe {
        if FREE_LIST_HEAD.is_null() || (*FREE_LIST_HEAD).next.is_null() {
            return;
        }
        
        let mut sorted: *mut FreeListBlock = null_mut();
        let mut current = FREE_LIST_HEAD;
        
        while !current.is_null() {
            let next = (*current).next;
            sorted = sorted_insert(sorted, current);
            current = next;
        }
        
        FREE_LIST_HEAD = sorted;
    }
}

unsafe fn sorted_insert(mut head: *mut FreeListBlock, new_block: *mut FreeListBlock) -> *mut FreeListBlock {
    unsafe {
        let new_addr = new_block as usize;
        
        // If list is empty or new block should be first
        if head.is_null() || (head as usize) > new_addr {
            (*new_block).next = head;
            return new_block;
        }
        
        // Find insertion point
        let mut current = head;
        while !(*current).next.is_null() && ((*current).next as usize) < new_addr {
            current = (*current).next;
        }
        
        (*new_block).next = (*current).next;
        (*current).next = new_block;
        
        head
    }
}

/// Merge adjacent free blocks
unsafe fn coalesce_free_list() {
    unsafe {
        let mut current = FREE_LIST_HEAD;
        
        while !current.is_null() {
            let current_addr = current as usize;
            let current_size = (*current).size;
            let next = (*current).next;
            
            if !next.is_null() {
                let next_addr = next as usize;
                
                // Check if blocks are adjacent
                if current_addr + current_size == next_addr {
                    // Merge blocks
                    (*current).size += (*next).size;
                    (*current).next = (*next).next;
                    // Continue checking from current block
                    continue;
                }
            }
            
            current = (*current).next;
        }
    }
}

/// Allocate a physical frame (4KB page)
/// Returns the physical address of the frame, or None if out of memory
pub unsafe fn alloc_frame() -> Option<u64> {
    unsafe {
        alloc_pages(1).map(|addr| addr as u64)
    }
}

/// Allocate multiple contiguous physical pages
/// Returns the physical address of the first page, or None if out of memory
pub unsafe fn alloc_pages(count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }

    let size_needed = count * PAGE_SIZE;
    
    unsafe {
        let mut prev: *mut *mut FreeListBlock = addr_of_mut!(FREE_LIST_HEAD);
        let mut current = FREE_LIST_HEAD;

        while !current.is_null() {
            let block_size = (*current).size;
            
            if block_size >= size_needed {
                let addr = current as usize;
                
                if block_size == size_needed {
                    // Perfect fit - remove entire block
                    *prev = (*current).next;
                } else if block_size - size_needed >= core::mem::size_of::<FreeListBlock>() {
                    // Split the block only if remainder is large enough
                    let new_block = (addr + size_needed) as *mut FreeListBlock;
                    (*new_block).size = block_size - size_needed;
                    (*new_block).next = (*current).next;
                    *prev = new_block;
                } else {
                    // Remainder too small, allocate entire block
                    *prev = (*current).next;
                }
                
                FREE_MEMORY -= size_needed;
                
                // Zero out the allocated memory
                let ptr = addr as *mut u8;
                for i in 0..size_needed {
                    *ptr.add(i) = 0;
                }
                
                return Some(addr);
            }
            
            prev = addr_of_mut!((*current).next);
            current = (*current).next;
        }
        
        None
    }
}

/// Free a physical frame
pub unsafe fn free_frame(addr: u64) {
    unsafe {
        free_pages(addr as usize, 1);
    }
}

/// Free multiple contiguous physical pages with coalescing
pub unsafe fn free_pages(addr: usize, count: usize) {
    if count == 0 {
        return;
    }

    let size = count * PAGE_SIZE;
    
    unsafe {
        // Find the correct insertion point to maintain sorted order
        let mut prev: *mut *mut FreeListBlock = addr_of_mut!(FREE_LIST_HEAD);
        let mut current = FREE_LIST_HEAD;
        
        // Find where this block should be inserted (sorted by address)
        while !current.is_null() && (current as usize) < addr {
            prev = addr_of_mut!((*current).next);
            current = (*current).next;
        }
        
        let new_block = addr as *mut FreeListBlock;
        (*new_block).size = size;
        (*new_block).next = current;
        *prev = new_block;
        
        FREE_MEMORY += size;
        
        // Try to merge with previous block
        if prev != addr_of_mut!(FREE_LIST_HEAD) {
            let prev_block = (prev as usize - core::mem::offset_of!(FreeListBlock, next)) as *mut FreeListBlock;
            let prev_addr = prev_block as usize;
            let prev_size = (*prev_block).size;
            
            if prev_addr + prev_size == addr {
                // Merge with previous block
                (*prev_block).size += size;
                (*prev_block).next = current;
                
                // Update new_block to point to merged block for next merge attempt
                let merged_block = prev_block;
                
                // Try to merge with next block
                if !current.is_null() {
                    let merged_addr = merged_block as usize;
                    let merged_size = (*merged_block).size;
                    let next_addr = current as usize;
                    
                    if merged_addr + merged_size == next_addr {
                        (*merged_block).size += (*current).size;
                        (*merged_block).next = (*current).next;
                    }
                }
                return;
            }
        }
        
        // Try to merge with next block
        if !current.is_null() && addr + size == (current as usize) {
            (*new_block).size += (*current).size;
            (*new_block).next = (*current).next;
        }
    }
}

/// Get total memory in bytes
pub unsafe fn get_total_memory() -> usize {
    unsafe {
        TOTAL_MEMORY
    }
}

/// Get free memory in bytes
pub unsafe fn get_free_memory() -> usize {
    unsafe {
        FREE_MEMORY
    }
}

/// Get used memory in bytes
pub unsafe fn get_used_memory() -> usize {
    unsafe {
        TOTAL_MEMORY - FREE_MEMORY
    }
}

/// Get memory statistics as (total, used, free)
pub unsafe fn get_stats() -> (usize, usize, usize) {
    unsafe {
        let total = TOTAL_MEMORY;
        let free = FREE_MEMORY;
        let used = total - free;
        (total, used, free)
    }
}