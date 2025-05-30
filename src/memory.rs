use core::ptr;

#[unsafe(no_mangle)]
pub unsafe fn memset(dest: *mut u8, value: u8, count: usize) {
    let mut ptr = dest;
    for _ in 0..count {
        unsafe { ptr::write(ptr, value) };
        ptr = unsafe { ptr.add(1) };
    }
}

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

#[unsafe(no_mangle)]
pub unsafe fn memcpy(dest: *mut u8, src: *const u8, count: usize) {
    for i in 0..count {
        unsafe { ptr::write(dest.add(i), ptr::read(src.add(i))) };
    }
}

#[unsafe(no_mangle)]
pub unsafe fn memmove(dest: *mut u8, src: *const u8, count: usize) {
    if dest as usize <= src as usize || dest as usize >= src as usize + count {
        // Non-overlapping regions, can copy forward
        for i in 0..count {
            unsafe{ ptr::write(dest.add(i), ptr::read(src.add(i))) };
        }
    } else {
        // Overlapping regions, copy backward
        for i in (0..count).rev() {
            unsafe{ ptr::write(dest.add(i), ptr::read(src.add(i))) };
        }
    }
}
