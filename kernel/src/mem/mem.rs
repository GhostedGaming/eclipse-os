//! A FIFO (First In, First Out) allocator implemented using a linked list.
//! Each block in the allocator is represented as a node in the list.

use core::{
    alloc::{GlobalAlloc, Layout},
    ffi::c_void,
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


// Tell rust to use this as the allocator
#[global_allocator]
static ALLOCATOR: LinkAllocator = LinkAllocator;

/// Initialize the allocator
pub unsafe fn init_allocator(memory_map: &MemoryMapResponse) {
    FREE_LIST = LinkedList::new();
    
    for entry in memory_map.entries() {
        if entry.entry_type == EntryType::USABLE && entry.length > 1024 * 1024 {
            HEAP_START = entry.base as *mut u8;
            HEAP_OFFSET = 0;
            break;
        }
    }
    
    if HEAP_START.is_null() {
        panic!("No usable memory found in memory map");
    }
}