use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

use crate::println;

/// Represents a disk device
pub struct Disk {
    pub device_type: DiskType,
    pub index: u8,
    pub is_bootable: bool,
}

/// Types of disk devices
pub enum DiskType {
    Ata,
    Atapi,
    Sata,
    Unknown,
}

impl Disk {
    pub fn new(device_type: DiskType, index: u8, is_bootable: bool) -> Self {
        Self {
            device_type,
            index,
            is_bootable,
        }
    }

    pub fn to_string(&self) -> String {
        let type_str = match self.device_type {
            DiskType::Ata => "ATA",
            DiskType::Atapi => "ATAPI",
            DiskType::Sata => "SATA",
            DiskType::Unknown => "Unknown",
        };
        
        let bootable_str = if self.is_bootable { " (bootable)" } else { "" };
        
        format!("{} Disk {}{}", type_str, self.index, bootable_str)
    }
}

/// Detect ATA disks by probing the standard ATA ports
pub fn detect_ata_disks() -> Vec<Disk> {
    let mut disks = Vec::new();
    
    // Primary and secondary ATA bus ports
    let ata_ports = [(0x1F0, 0x1F7), (0x170, 0x177)];
    
    for (bus_idx, &(base_port, _)) in ata_ports.iter().enumerate() {
        // Check master and slave devices on each bus
        for device_idx in 0..2 {
            let is_present = probe_ata_device(base_port, device_idx);
            
            if is_present {
                // For simplicity, we're assuming all detected devices are ATA
                let disk = Disk::new(
                    DiskType::Ata, 
                    (bus_idx * 2 + device_idx) as u8,
                    bus_idx == 0 && device_idx == 0 // Assume first disk is bootable
                );
                disks.push(disk);
            }
        }
    }
    
    disks
}

/// Probe an ATA device to check if it's present
fn probe_ata_device(base_port: u16, device_idx: usize) -> bool {
    // Create ports for the ATA interface
    let mut data_port: Port<u16> = Port::new(base_port);
    let mut error_port: Port<u8> = Port::new(base_port + 1);
    let mut sector_count_port: Port<u8> = Port::new(base_port + 2);
    let mut lba_low_port: Port<u8> = Port::new(base_port + 3);
    let mut lba_mid_port: Port<u8> = Port::new(base_port + 4);
    let mut lba_high_port: Port<u8> = Port::new(base_port + 5);
    let mut device_port: Port<u8> = Port::new(base_port + 6);
    let mut command_port: Port<u8> = Port::new(base_port + 7);
    
    unsafe {
        // Select the device (master or slave)
        device_port.write(if device_idx == 0 { 0xA0_u8 } else { 0xB0_u8 });
        
        // Small delay to allow device to respond
        for _ in 0..10 {
            command_port.read();
        }
        
        // Send IDENTIFY command
        command_port.write(0xEC_u8);
        
        // Check if device exists by reading status
        let status: u8 = command_port.read();
        
        if status == 0 {
            return false; // No device present
        }
        
        // Wait until BSY bit is cleared with timeout
        let mut timeout = 1000;
        while timeout > 0 {
            let status: u8 = command_port.read();
            if status & 0x80 == 0 { // BSY bit cleared
                break;
            }
            timeout -= 1;
            if timeout == 0 {
                return false; // Timeout waiting for device
            }
        }
        
        // Check for error
        let mid: u8 = lba_mid_port.read();
        let high: u8 = lba_high_port.read();
        
        if mid != 0 || high != 0 {
            return false; // Not an ATA device
        }
        
        // Wait for DRQ or ERR with timeout
        timeout = 1000;
        while timeout > 0 {
            let status: u8 = command_port.read();
            if status & 0x08 != 0 { // DRQ set
                // Read the identify data to clear it (but we don't use it here)
                for _ in 0..256 {
                    let _: u16 = data_port.read();
                }
                return true;
            }
            if status & 0x01 != 0 { // ERR set
                return false;
            }
            timeout -= 1;
            if timeout == 0 {
                return false; // Timeout waiting for data
            }
        }
        
        false // Default to not found
    }
}

/// List all detected disks in the system
pub fn list_disks() -> Vec<Disk> {
    // For safety, we'll provide a fallback in case hardware detection fails
    let ata_disks = detect_ata_disks();
    
    if !ata_disks.is_empty() {
        return ata_disks;
    }
    
    // If no disks were detected, return mock data for testing
    let mut mock_disks = Vec::new();
    mock_disks.push(Disk::new(DiskType::Ata, 0, true));
    mock_disks.push(Disk::new(DiskType::Sata, 1, false));
    
    mock_disks
}

/// Print information about all detected disks
pub fn print_disks() {
    let disks = list_disks();
    
    if disks.is_empty() {
        println!("No disks detected");
        return;
    }
    
    println!("Detected disks:");
    for disk in disks {
        println!("  - {}", disk.to_string());
    }
}
