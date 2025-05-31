use core::ptr;

/// Sets `count` bytes starting at `dest` to the given `value`.
///
/// # Safety
///
/// - `dest` must be valid for writes of `count` bytes.
/// - The memory regions must not overlap with any other references for the duration of this call.
/// - Behavior is undefined if `dest` is null or not properly aligned.
#[unsafe(no_mangle)]
pub unsafe fn memset(dest: *mut u8, value: u8, count: usize) {
    let mut ptr = dest;
    for _ in 0..count {
        unsafe { ptr::write(ptr, value) };
        ptr = unsafe { ptr.add(1) };
    }
}

/// Compares the first `n` bytes of the memory areas `s1` and `s2`.
///
/// # Safety
///
/// - Both `s1` and `s2` must be valid for reads of `n` bytes.
/// - The memory regions must not overlap with any mutable references for the duration of this call.
/// - Behavior is undefined if either pointer is null or not properly aligned.
#[unsafe(no_mangle)]
pub unsafe fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let a = unsafe { ptr::read(s1.add(i)) };
        let b = unsafe { ptr::read(s2.add(i)) };
        if a != b {
            return a as i32 - b as i32;
        }
    }
    0
}

/// Copies `count` bytes from `src` to `dest`.
///
/// # Safety
///
/// - Both `dest` and `src` must be valid for reads and writes of `count` bytes.
/// - The memory regions must not overlap with any other references for the duration of this call.
/// - Behavior is undefined if either pointer is null or not properly aligned.
#[unsafe(no_mangle)]
pub unsafe fn memcpy(dest: *mut u8, src: *const u8, count: usize) {
    for i in 0..count {
        unsafe { ptr::write(dest.add(i), ptr::read(src.add(i))) };
    }
}

/// Moves `count` bytes from `src` to `dest`, correctly handling overlapping regions.
///
/// # Safety
///
/// - Both `dest` and `src` must be valid for reads and writes of `count` bytes.
/// - The memory regions must not overlap with any other references for the duration of this call.
/// - Behavior is undefined if either pointer is null or not properly aligned.
#[unsafe(no_mangle)]
pub unsafe fn memmove(dest: *mut u8, src: *const u8, count: usize) {
    if dest as usize <= src as usize || dest as usize >= src as usize + count {
        // Non-overlapping regions, can copy forward
        for i in 0..count {
            unsafe { ptr::write(dest.add(i), ptr::read(src.add(i))) };
        }
    } else {
        // Overlapping regions, copy backward
        for i in (0..count).rev() {
            unsafe { ptr::write(dest.add(i), ptr::read(src.add(i))) };
        }
    }
}
