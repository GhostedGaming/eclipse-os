#![no_std]

#[macro_export]
macro_rules! inb {
    ($port:expr) => {{
        let result: u8;
        unsafe {
            core::arch::asm!(
                "in al, dx",
                in("dx") $port,
                out("al") result,
                options(nomem, nostack, preserves_flags)
            );
        }
        result
    }};
}

#[macro_export]
macro_rules! outb {
    ($port:expr, $value:expr) => {{
        let port: u16 = $port;
        let value: u8 = $value;
        unsafe {
            core::arch::asm!(
                "out dx, al",
                in("dx") port,
                in("al") value,
                options(nomem, nostack, preserves_flags)
            );
        }
    }};
}

#[macro_export]
macro_rules! inw {
    ($port:expr) => {{
        let result: u16;
        unsafe {
            core::arch::asm!(
                "in ax, dx",
                in("dx") $port,
                out("ax") result,
                options(nomem, nostack, preserves_flags)
            );
        }
        result
    }};
}

#[macro_export]
macro_rules! outw {
    ($port:expr, $value:expr) => {{
        let port: u16 = $port;
        let value: u16 = $value;
        unsafe {
            core::arch::asm!(
                "out dx, ax",
                in("dx") port,
                in("ax") value,
                options(nomem, nostack, preserves_flags)
            );
        }
    }};
}

#[macro_export]
macro_rules! inl {
    ($port:expr) => {{
        let result: u32;
        unsafe {
            core::arch::asm!(
                "in eax, dx",
                in("dx") $port,
                out("eax") result,
                options(nomem, nostack, preserves_flags)
            );
        }
        result
    }};
}

#[macro_export]
macro_rules! outl {
    ($port:expr, $value:expr) => {{
        let port: u16 = $port;
        let value: u32 = $value;
        unsafe {
            core::arch::asm!(
                "out dx, eax",
                in("dx") port,
                in("eax") value,
                options(nomem, nostack, preserves_flags)
            );
        }
    }};
}