use crate::println;

#[macro_export]
// Print to com1
macro_rules! port_println {
    ($($arg:tt)*) => {{
        use x86_64::instructions::port::Port;
        let port_addr = 0x3F8; // COM1
        let mut port = Port::<u8>::new(port_addr);
        let s = alloc::format!($($arg)*);
        for byte in s.as_bytes() {
            unsafe { port.write(*byte); }
        }
        unsafe { port.write(b'\n'); }
    }};
}

// Read the usb port
pub fn read_port() {
    use x86_64::instructions::port::Port;
    let port_adr = 0x3F8;
    let mut port = Port::<u8>::new(port_adr);
    println!("{:?}", unsafe { port.read() });
}