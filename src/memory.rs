use alloc::vec::Vec;
use uefi::{
    boot::{self, MemoryDescriptor, MemoryType, memory_map},
    mem::memory_map::{MemoryMap, MemoryMapOwned},
};
// use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
};

/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

/// A FrameAllocator that always returns `None`.
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: *const MemoryMapOwned,
    next: usize,
}

pub const PHYSICAL_MEMORY_OFFSET: u64 = 0xffff_8000_0000_0000;

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the passed memory map is valid.
    /// The main requirement is that all frames that are marked as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map_owned: &MemoryMapOwned) -> impl FrameAllocator<Size4KiB> {
        let physical_memory_offset = VirtAddr::new(PHYSICAL_MEMORY_OFFSET);
        let level_4_table = unsafe { active_level_4_table(physical_memory_offset) };
        unsafe { OffsetPageTable::new(level_4_table, physical_memory_offset); }

        let memory_map = memory_map_owned as *const MemoryMapOwned;
        let next = 0;
        BootInfoFrameAllocator {
            memory_map,
            next,
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // SAFETY: The caller must guarantee that the memory map reference outlives this allocator.
        let memory_map = unsafe { &*self.memory_map };
        // get usable regions from memory map
        let regions = memory_map.entries();
        let usable_regions = regions.filter(|r| r.ty == MemoryType::CONVENTIONAL);
        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| {
            let start_addr = r.phys_start;
            let end_addr = r.phys_start + r.page_count * 4096;
            start_addr..end_addr
        });
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
