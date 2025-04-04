use core::arch::asm;
use alloc::vec::Vec;
use alloc::string::String;
use crate::println;

const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

/// Write to an I/O port
unsafe fn outl(port: u16, value: u32) {
    asm!("out dx, eax", in("dx") port, in("eax") value);
}

/// Read from an I/O port
pub unsafe fn inl(port: u16) -> u32 {
    let value: u32;
    asm!("in eax, dx", out("eax") value, in("dx") port);
    value
}

/// Create a PCI configuration address
fn pci_config_address(bus: u8, slot: u8, function: u8, offset: u8) -> u32 {
    0x80000000 | ((bus as u32) << 16) | ((slot as u32) << 11) | ((function as u32) << 8) | ((offset as u32) & 0xFC)
}

/// Read a 32-bit value from PCI configuration space
fn pci_read(bus: u8, slot: u8, function: u8, offset: u8) -> u32 {
    let address = pci_config_address(bus, slot, function, offset);
    unsafe {
        outl(PCI_CONFIG_ADDRESS, address);
        inl(PCI_CONFIG_DATA)
    }
}

/// Check if a PCI device exists
fn pci_device_exists(bus: u8, slot: u8) -> bool {
    let vendor_id = pci_read(bus, slot, 0, 0) & 0xFFFF;
    vendor_id != 0xFFFF
}

/// Enumerate PCI devices
pub fn enumerate_pci_devices() -> Vec<(u8, u8, u8)> {
    let mut devices = Vec::new();
    for bus in 0..255 {
        for slot in 0..32 {
            for function in 0..8 {
                if pci_device_exists(bus, slot) {
                    devices.push((bus, slot, function));
                }
            }
        }
    }
    devices
}

/// Main function to list disks
pub fn list_disks() {
    let devices = enumerate_pci_devices();

    println!("PCI Devices:");
    println!("------------");

    for (bus, slot, function) in devices.iter() {
        let vendor_id = pci_read(*bus, *slot, *function, 0) & 0xFFFF;
        let device_id = (pci_read(*bus, *slot, *function, 0) >> 16) & 0xFFFF;
        println!("Bus: {}, Slot: {}, Function: {}, Vendor ID: 0x{:04x}, Device ID: 0x{:04x}", bus, slot, function, vendor_id, device_id);
        
        // Check if it is a storage controller and initialize it
        let class_code = (pci_read(*bus, *slot, *function, 8) >> 24) & 0xFF;
        let subclass = (pci_read(*bus, *slot, *function, 8) >> 16) & 0xFF;
        
        if class_code == 0x01 { // Mass Storage Controller
            match subclass {
                0x01 => println!("Found IDE Controller"),
                0x06 => println!("Found SATA Controller"),
                0x08 => println!("Found NVMe Controller"),
                _ => println!("Found Other Storage Controller"),
            }
        }
    }
}
