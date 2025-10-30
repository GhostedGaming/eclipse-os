use core::ptr::{addr_of_mut, null_mut};
use limine::memory_map::EntryType;
use limine::response::MemoryMapResponse;

const PAGE_SIZE: usize = 4096;

#[repr(C)]
struct FreeList {
    size: usize,
    next: *mut FreeList,
}

static mut FREE_LIST_HEAD: *mut FreeList = null_mut();
static mut TOTAL_MEMORY: usize = 0;
static mut FREE_MEMORY: usize = 0;


fn pmm_init() {
    unsafe { 

    } 
}