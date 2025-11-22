#![no_std]
//! This file is for initializing and writing to AHCI (Advanced Host Controller Interface) drives
//! In 2004 Intel created AHCI to replace the older Parallel ATA (PATA) interface
//! AHCI provided native command queuing hot-plug support and better performance
//! It became the standard for SATA (Serial ATA) controllers on modern systems
//! AHCI controllers support multiple ports and provide a more efficient way to manage SATA devices

extern crate alloc;

use eclipse_framebuffer::println;
use pci::{pci_config_read_dword, PCI_CLASS_MASS_STORAGE, PCI_SUBCLASS_SATA};

pub use types::*;
mod types;

fn start_cmd(port: &mut HbaPort) {
    let cmd = port.read_cmd();
    if (cmd & (1 << 4)) != 0 {
        return;
    }
    while (cmd & (1 << 15)) != 0 {
        // Wait until CR (bit15) is cleared
    }
    port.write_cmd(cmd | (1 << 4));
}

fn stop_cmd(port: &mut HbaPort) {
    let cmd = port.read_cmd();
    port.write_cmd(cmd & !(1 << 4));
    while (cmd & (1 << 15)) != 0 {
        // Wait until CR (bit15) is cleared
    }
    port.write_cmd(cmd & !(1 << 0));
    while (cmd & (1 << 14)) != 0 {
        // Wait until FR (bit14) is cleared
    }
}

fn rebase_port(port: &mut HbaPort, portno: u32, base: u64) {
    stop_cmd(port);
    
    port.clb = base + ((portno as u64) << 10);
    unsafe { core::ptr::write_bytes(port.clb as *mut u8, 0, 1024); }
    
    port.fb = base + (32 << 10) + ((portno as u64) << 8);
    unsafe { core::ptr::write_bytes(port.fb as *mut u8, 0, 256); }
    

    let cmdheader = port.clb as *mut HbaCmdHeader;
    for i in 0..32 {
        unsafe {
            (*cmdheader.add(i)).prdtl = 8;
            
            (*cmdheader.add(i)).ctba = base + (40 << 10) + ((portno as u64) << 13) + ((i as u64) << 8);
            
            core::ptr::write_bytes((*cmdheader.add(i)).ctba as *mut u8, 0, 256);
        }
    }
    
    start_cmd(port);
}

pub fn probe_ports(abar: &mut HbaMem) {
    let pi = abar.read_pi();
    
    for i in 0..32 {
        if (pi >> i) & 1 != 0 {
            let dt = check_type(&abar.ports[i]);
            match dt {
                AHCI_DEV_SATA => {
                    println!("SATA drive found at port {}", i);
                    rebase_port(&mut abar.ports[i], i as u32, 0x400000);
                }
                AHCI_DEV_SATAPI => {
                    println!("SATAPI drive found at port {}", i);
                    rebase_port(&mut abar.ports[i], i as u32, 0x400000);
                }
                AHCI_DEV_SEMB => {
                    println!("SEMB drive found at port {}", i);
                }
                AHCI_DEV_PM => {
                    println!("PM drive found at port {}", i);
                }
                _ => {
                    println!("No drive found at port {}", i);
                }
            }
        }
    }
}

fn check_type(port: &HbaPort) -> u8 {
    let ssts = port.read_ssts();
    let ipm = (ssts >> 8) & 0x0F;
    let det = ssts & 0x0F;
    
    if det != HBA_PORT_DET_PRESENT {
        return AHCI_DEV_NULL;
    }
    if ipm != HBA_PORT_IPM_ACTIVE {
        return AHCI_DEV_NULL;
    }
    
    match port.read_sig() {
        HBA_PORT_SIG_ATAPI => AHCI_DEV_SATAPI,
        HBA_PORT_SIG_SEMB => AHCI_DEV_SEMB,
        HBA_PORT_SIG_PM => AHCI_DEV_PM,
        _ => AHCI_DEV_SATA,
    }
}

pub fn find_ahci_controller() -> Option<u64> {
    
    println!("Scanning PCI for AHCI controller...");
    
    for bus in 0..=255u16 {
        for device in 0..32u8 {
            for function in 0..8u8 {
                let vendor_id = pci_config_read_dword(bus as u8, device, function, 0x00) & 0xFFFF;
                
                if vendor_id == 0xFFFF || vendor_id == 0x0000 {
                    continue;
                }
                
                let class_reg = pci_config_read_dword(bus as u8, device, function, 0x08);
                let class_code = (class_reg >> 24) & 0xFF;
                let subclass = (class_reg >> 16) & 0xFF;
                let prog_if = (class_reg >> 8) & 0xFF;
                
                if class_code == PCI_CLASS_MASS_STORAGE as u32 && 
                   subclass == PCI_SUBCLASS_SATA as u32 && 
                   prog_if == 0x01 {
                    println!("Found AHCI controller at {}:{}:{}", bus, device, function);
                    let bar5 = pci_config_read_dword(bus as u8, device, function, 0x24);
                    let abar = (bar5 & !0xF) as u64;
                    println!("BAR5 = 0x{:X}", abar);
                    return Some(abar);
                }
            }
        }
    }
    
    println!("No AHCI controller found");
    None
}

pub fn ahci_read(port: &HbaPort, lba: u64, count: u32, buffer: *mut u8) -> bool {
    let ci = port.read_ci();
    if ci != 0 {
        return false;
    }

    let cmdheader = port.clb as *mut HbaCmdHeader;
    unsafe {
        (*cmdheader).prdtl = 1;
        
        let cmdtbl = (*cmdheader).ctba as *mut HbaCmdTbl;
        core::ptr::write_bytes(cmdtbl as *mut u8, 0, 256);
        
        let fis = &mut (*cmdtbl).cfis;
        fis[0] = 0x27;
        fis[1] = 0x80;
        fis[2] = 0xC8;
        fis[3] = 0x00;
        fis[4] = (lba & 0xFF) as u8;
        fis[5] = ((lba >> 8) & 0xFF) as u8;
        fis[6] = ((lba >> 16) & 0xFF) as u8;
        fis[7] = 0xE0 | ((lba >> 24) & 0x0F) as u8;
        fis[8] = ((lba >> 32) & 0xFF) as u8;
        fis[9] = ((lba >> 40) & 0xFF) as u8;
        fis[10] = ((lba >> 48) & 0xFF) as u8;
        fis[11] = 0x00;
        fis[12] = (count & 0xFF) as u8;
        fis[13] = ((count >> 8) & 0xFF) as u8;
        
        (*cmdtbl).prdt_entry[0].dba = buffer as u64;
        (*cmdtbl).prdt_entry[0].dbc = (count as u32 * 512) - 1;
        
        let port_mut = port as *const HbaPort as *mut HbaPort;
        (*port_mut).ci = 1;
        
        let mut timeout = 1000000;
        while ((*port_mut).ci & 1) != 0 && timeout > 0 {
            timeout -= 1;
        }
    }
    
    true
}

pub fn ahci_write(port: &HbaPort, lba: u64, count: u32, buffer: *const u8) -> bool {
    let ci = port.read_ci();
    if ci != 0 {
        return false;
    }

    let cmdheader = port.clb as *mut HbaCmdHeader;
    unsafe {
        (*cmdheader).prdtl = 1;
        
        let cmdtbl = (*cmdheader).ctba as *mut HbaCmdTbl;
        core::ptr::write_bytes(cmdtbl as *mut u8, 0, 256);
        
        let fis = &mut (*cmdtbl).cfis;
        fis[0] = 0x27;
        fis[1] = 0x80;
        fis[2] = 0xCA;
        fis[3] = 0x00;
        fis[4] = (lba & 0xFF) as u8;
        fis[5] = ((lba >> 8) & 0xFF) as u8;
        fis[6] = ((lba >> 16) & 0xFF) as u8;
        fis[7] = 0xE0 | ((lba >> 24) & 0x0F) as u8;
        fis[8] = ((lba >> 32) & 0xFF) as u8;
        fis[9] = ((lba >> 40) & 0xFF) as u8;
        fis[10] = ((lba >> 48) & 0xFF) as u8;
        fis[11] = 0x00;
        fis[12] = (count & 0xFF) as u8;
        fis[13] = ((count >> 8) & 0xFF) as u8;
        
        (*cmdtbl).prdt_entry[0].dba = buffer as u64;
        (*cmdtbl).prdt_entry[0].dbc = (count as u32 * 512) - 1;
        
        let port_mut = port as *const HbaPort as *mut HbaPort;
        (*port_mut).ci = 1;
        
        let mut timeout = 1000000;
        while ((*port_mut).ci & 1) != 0 && timeout > 0 {
            timeout -= 1;
        }
    }
    
    true
}