//! This file defines all of the PCI functions
//! What is PCI? PCI (Peripheral Component Interconnect) is a local bus used to connect
//! hardware to a computers motherboard 

extern crate alloc;

use eclipse_x86_64_commands::{outl, inl};
use eclipse_framebuffer::println;
use alloc::boxed::Box;

// PCI Configuration Space I/O Ports
const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

// PCI Limits
const MAX_PCI_DEVICES: usize = 256;

// PCI Configuration Space Offsets
const PCI_VENDOR_ID: u8 = 0x00;
const PCI_DEVICE_ID: u8 = 0x02;
const PCI_COMMAND: u8 = 0x04;
const PCI_CLASS_CODE: u8 = 0x0B;
const PCI_SUBCLASS: u8 = 0x0A;
const PCI_PROG_IF: u8 = 0x09;
const PCI_HEADER_TYPE: u8 = 0x0E;
const PCI_BAR0: u8 = 0x10;
const PCI_SECONDARY_BUS: u8 = 0x19;
const PCI_INTERRUPT_LINE: u8 = 0x3C;
const PCI_INTERRUPT_PIN: u8 = 0x3D;

// PCI Class Codes
const PCI_CLASS_BRIDGE: u8 = 0x06;
const PCI_SUBCLASS_PCI_BRIDGE: u8 = 0x04;

// PCI Mass Storage
const PCI_CLASS_MASS_STORAGE: u8 = 0x01;
const PCI_SUBCLASS_SATA: u8 = 0x06;
const PCI_PROG_IF_AHCI: u8 = 0x01;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PCIDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub bar: [u32; 6],
    pub interrupt_line: u8,
}

impl PCIDevice {
    const fn new() -> Self {
        Self {
            bus: 0,
            device: 0,
            function: 0,
            vendor_id: 0,
            device_id: 0,
            class_code: 0,
            subclass: 0,
            prog_if: 0,
            bar: [0; 6],
            interrupt_line: 0,
        }
    }
}

static mut PCI_DEVICES: [PCIDevice; MAX_PCI_DEVICES] = [PCIDevice::new(); MAX_PCI_DEVICES];
static mut PCI_DEVICE_COUNT: u32 = 0;

pub async fn pci_config_read_dword(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address: u32 = ((bus as u32) << 16) 
        | ((device as u32) << 11)
        | ((function as u32) << 8) 
        | ((offset as u32) & 0xFC) 
        | 0x80000000;
    
    outl!(PCI_CONFIG_ADDRESS, address);
    inl!(PCI_CONFIG_DATA)
}

pub async fn pci_config_write_dword(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
    let address: u32 = ((bus as u32) << 16) 
        | ((device as u32) << 11)
        | ((function as u32) << 8) 
        | ((offset as u32) & 0xFC) 
        | 0x80000000;
    
    outl!(PCI_CONFIG_ADDRESS, address);
    outl!(PCI_CONFIG_DATA, value);
}

pub async fn pci_config_read_word(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    let data: u32 = pci_config_read_dword(bus, device, function, offset).await;
    let shift: u8 = if (offset & 2) != 0 { 16 } else { 0 };
    ((data >> shift) & 0xFFFF) as u16
}

pub async fn pci_config_read_byte(bus: u8, device: u8, function: u8, offset: u8) -> u8 {
    let data: u32 = pci_config_read_dword(bus, device, function, offset).await;
    let shift: u8 = (offset & 3) * 8;
    ((data >> shift) & 0xFF) as u8
}

pub async fn pci_config_write_word(bus: u8, device: u8, function: u8, offset: u8, value: u16) {
    let mut data: u32 = pci_config_read_dword(bus, device, function, offset).await;
    let shift: u8 = (offset & 2) * 8;
    data &= !(0xFFFF << shift);
    data |= (value as u32) << shift;
    pci_config_write_dword(bus, device, function, offset, data).await;
}

pub async fn pci_config_write_byte(bus: u8, device: u8, function: u8, offset: u8, value: u8) {
    let mut data: u32 = pci_config_read_dword(bus, device, function, offset).await;
    let shift: u8 = (offset & 3) * 8;
    data &= !(0xFF << shift);
    data |= (value as u32) << shift;
    pci_config_write_dword(bus, device, function, offset, data).await;
}

pub async fn get_vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    pci_config_read_word(bus, device, function, PCI_VENDOR_ID).await
}

pub async fn get_device_id(bus: u8, device: u8, function: u8) -> u16 {
    pci_config_read_word(bus, device, function, PCI_DEVICE_ID).await
}

pub async fn pci_read_bar(bus: u8, device: u8, function: u8, bar_num: u8) -> u32 {
    pci_config_read_dword(bus, device, function, PCI_BAR0 + (bar_num * 4)).await
}

pub async fn pci_write_bar(bus: u8, device: u8, function: u8, bar_num: u8, value: u32) {
    pci_config_write_dword(bus, device, function, PCI_BAR0 + (bar_num * 4), value).await;
}

pub async fn pci_get_bar_size(bus: u8, device: u8, function: u8, bar_num: u8) -> u32 {
    let original = pci_read_bar(bus, device, function, bar_num).await;
    pci_write_bar(bus, device, function, bar_num, 0xFFFFFFFF).await;
    let mut size = pci_read_bar(bus, device, function, bar_num).await;
    pci_write_bar(bus, device, function, bar_num, original).await;
    
    if (original & 0x1) != 0 {
        size &= 0xFFFFFFFC;
    } else {
        size &= 0xFFFFFFF0;
    }
    
    (!size).wrapping_add(1)
}

pub async fn pci_enable_bus_master(bus: u8, device: u8, function: u8) {
    let mut command = pci_config_read_word(bus, device, function, PCI_COMMAND).await;
    command |= 0x04;
    pci_config_write_word(bus, device, function, PCI_COMMAND, command).await;
}

pub async fn pci_disable_bus_master(bus: u8, device: u8, function: u8) {
    let mut command = pci_config_read_word(bus, device, function, PCI_COMMAND).await;
    command &= !0x04;
    pci_config_write_word(bus, device, function, PCI_COMMAND, command).await;
}

pub async fn pci_enable_memory_space(bus: u8, device: u8, function: u8) {
    let mut command = pci_config_read_word(bus, device, function, PCI_COMMAND).await;
    command |= 0x02;
    pci_config_write_word(bus, device, function, PCI_COMMAND, command).await;
}

pub async fn pci_enable_io_space(bus: u8, device: u8, function: u8) {
    let mut command = pci_config_read_word(bus, device, function, PCI_COMMAND).await;
    command |= 0x01;
    pci_config_write_word(bus, device, function, PCI_COMMAND, command).await;
}

pub async fn pci_get_interrupt_line(bus: u8, device: u8, function: u8) -> u8 {
    pci_config_read_byte(bus, device, function, PCI_INTERRUPT_LINE).await
}

pub async fn pci_get_interrupt_pin(bus: u8, device: u8, function: u8) -> u8 {
    pci_config_read_byte(bus, device, function, PCI_INTERRUPT_PIN).await
}

pub async fn pci_add_device(bus: u8, device: u8, function: u8) {
    unsafe {
        if PCI_DEVICE_COUNT >= MAX_PCI_DEVICES as u32 {
            return;
        }

        let dev = &mut PCI_DEVICES[PCI_DEVICE_COUNT as usize];
        dev.bus = bus;
        dev.device = device;
        dev.function = function;
        dev.vendor_id = get_vendor_id(bus, device, function).await;
        dev.device_id = get_device_id(bus, device, function).await;
        dev.class_code = pci_config_read_byte(bus, device, function, PCI_CLASS_CODE).await;
        dev.subclass = pci_config_read_byte(bus, device, function, PCI_SUBCLASS).await;
        dev.prog_if = pci_config_read_byte(bus, device, function, PCI_PROG_IF).await;
        dev.interrupt_line = pci_get_interrupt_line(bus, device, function).await;

        for i in 0..6 {
            dev.bar[i] = pci_read_bar(bus, device, function, i as u8).await;
        }

        PCI_DEVICE_COUNT += 1;
    }
}

pub async fn pci_find_device(vendor_id: u16, device_id: u16) -> Option<&'static PCIDevice> {
    unsafe {
        for i in 0..PCI_DEVICE_COUNT as usize {
            if PCI_DEVICES[i].vendor_id == vendor_id && PCI_DEVICES[i].device_id == device_id {
                return Some(&PCI_DEVICES[i]);
            }
        }
    }
    None
}

pub async fn pci_find_class(class_code: u8, subclass: u8) -> Option<&'static PCIDevice> {
    unsafe {
        for i in 0..PCI_DEVICE_COUNT as usize {
            if PCI_DEVICES[i].class_code == class_code && PCI_DEVICES[i].subclass == subclass {
                return Some(&PCI_DEVICES[i]);
            }
        }
    }
    None
}

pub async fn pci_find_class_prog_if(class_code: u8, subclass: u8, prog_if: u8) -> Option<&'static PCIDevice> {
    unsafe {
        for i in 0..PCI_DEVICE_COUNT as usize {
            if PCI_DEVICES[i].class_code == class_code 
                && PCI_DEVICES[i].subclass == subclass
                && PCI_DEVICES[i].prog_if == prog_if {
                return Some(&PCI_DEVICES[i]);
            }
        }
    }
    None
}

pub async fn check_function(bus: u8, device: u8, function: u8) {
    let vendor = get_vendor_id(bus, device, function).await;
    if vendor == 0xFFFF {
        return;
    }

    let device_id = get_device_id(bus, device, function).await;
    println!("Found PCI device: Bus {:02x}, Device {:02x}, Func {:02x} => Vendor: {:04x}, Device: {:04x}",
             bus, device, function, vendor, device_id);

    pci_add_device(bus, device, function).await;

    let base_class = pci_config_read_byte(bus, device, function, PCI_CLASS_CODE).await;
    let sub_class = pci_config_read_byte(bus, device, function, PCI_SUBCLASS).await;

    if base_class == PCI_CLASS_BRIDGE && sub_class == PCI_SUBCLASS_PCI_BRIDGE {
        let secondary_bus = pci_config_read_byte(bus, device, function, PCI_SECONDARY_BUS).await;
        Box::pin(check_bus(secondary_bus)).await;
    }
}

pub async fn check_device(bus: u8, device: u8) {
    let vendor = get_vendor_id(bus, device, 0).await;
    if vendor == 0xFFFF {
        return;
    }

    Box::pin(check_function(bus, device, 0)).await;

    let header_type = pci_config_read_byte(bus, device, 0, PCI_HEADER_TYPE).await;
    if (header_type & 0x80) != 0 {
        for function in 1..8 {
            if get_vendor_id(bus, device, function).await != 0xFFFF {
                Box::pin(check_function(bus, device, function)).await;
            }
        }
    }
}

pub async fn check_bus(bus: u8) {
    for device in 0..32 {
        Box::pin(check_device(bus, device)).await;
    }
}

pub async fn check_all_buses() {
    let header_type = pci_config_read_byte(0, 0, 0, PCI_HEADER_TYPE).await;
    if (header_type & 0x80) == 0 {
        Box::pin(check_bus(0)).await;
    } else {
        for function in 0..8 {
            if get_vendor_id(0, 0, function).await != 0xFFFF {
                Box::pin(check_bus(function)).await;
            }
        }
    }
}

pub async fn pci_find_ahci_controller() -> Option<&'static PCIDevice> {
    pci_find_class_prog_if(PCI_CLASS_MASS_STORAGE, PCI_SUBCLASS_SATA, PCI_PROG_IF_AHCI).await
}

pub async fn pci_read_word(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
    pci_config_read_word(bus, slot, func, offset).await
}

pub async fn pci_read_byte(bus: u8, slot: u8, func: u8, offset: u8) -> u8 {
    pci_config_read_byte(bus, slot, func, offset).await
}

pub async fn pci_read_dword(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    pci_config_read_dword(bus, slot, func, offset).await
}