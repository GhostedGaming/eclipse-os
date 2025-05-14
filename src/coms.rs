use alloc::vec::Vec;
use x86_64::instructions::port::Port;

use crate::println;

pub fn init_ports() {
    // Create an array of port addresses
    let port_addresses = [0x3F8, 0x2F8, 0x3E8, 0x2E8];
    let write_bytes = [b'H', b'e', b'l', b'l'];
    let mut result: Vec<u8> = Vec::new();

    // Write to each port
    for (addr, byte) in port_addresses.iter().zip(write_bytes.iter()) {
        let mut port = Port::<u8>::new(*addr);
        unsafe { port.write(*byte); }
    }

    // Read from each port and push into result
    for addr in port_addresses.iter() {
        let mut port = Port::<u8>::new(*addr);
        let byte = unsafe { port.read() };
        result.push(byte);
    }

    println!("Port read result: {:?}", result);
}