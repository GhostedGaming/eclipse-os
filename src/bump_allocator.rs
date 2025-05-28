use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct BumpAllocator<const HEAP_SIZE: usize> {
    pub heap: UnsafeCell<[MaybeUninit<u8>; HEAP_SIZE]>,
    pub offset: AtomicUsize,
}

impl<const HEAP_SIZE: usize> BumpAllocator<HEAP_SIZE> {
    pub const fn new() -> Self {
        Self {
            heap: UnsafeCell::new([MaybeUninit::uninit(); HEAP_SIZE]),
            offset: AtomicUsize::new(0),
        }
    }
}

unsafe impl<const HEAP_SIZE: usize> GlobalAlloc for BumpAllocator<HEAP_SIZE> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        let heap_start = self.heap.get().cast::<u8>();
        loop {
            let orig_offset = self.offset.load(Ordering::Relaxed);
            let ptr = unsafe { heap_start.add(orig_offset) };

            let offset = ptr.align_offset(align);
            if offset == usize::MAX {
                return core::ptr::null_mut();
            }

            let alloc = unsafe { ptr.add(offset) };
            if unsafe { alloc.offset_from(heap_start) } as usize + size > HEAP_SIZE {
                return core::ptr::null_mut();
            }

            if self
                .offset
                .compare_exchange_weak(
                    orig_offset,
                    orig_offset + offset + size,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                return alloc;
            } else {
                continue;
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        let heap_start = self.heap.get().cast::<u8>();
        let start_of_alloc = unsafe { ptr.offset_from(heap_start) } as usize;
        let end_of_alloc = unsafe { ptr.add(size).offset_from(heap_start) } as usize;

        let _ = self.offset.compare_exchange(
            end_of_alloc,
            start_of_alloc,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
    }
}

unsafe impl<const HEAP_SIZE: usize> Sync for BumpAllocator<HEAP_SIZE> {}
