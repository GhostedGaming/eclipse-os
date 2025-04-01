use x86_64::instructions::port::Port;

pub fn shutdown() {
    let mut port = Port::new(0x604);
    unsafe {
        port.write(0x2000_u16);
    }
}
