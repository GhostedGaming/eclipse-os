#![no_std]
//! This file is for initializng and writing to IDE drives
//! In 1986 Western Digital and Compaq created a new drive
//! Called the ata drive which replaced old storage devices
//! It was also known as IDE
extern crate alloc;

use alloc::vec::Vec;
use core::slice;
use core::mem::size_of_val;
use eclipse_framebuffer::println;
use eclipse_x86_64_commands::*;
use spin::Mutex;

const ATA_SR_BSY: u8 = 0x80;
const ATA_SR_DRDY: u8 = 0x40;
const ATA_SR_DF: u8 = 0x20;
const ATA_SR_DSC: u8 = 0x10;
const ATA_SR_DRQ: u8 = 0x08;
const ATA_SR_CORR: u8 = 0x04;
const ATA_SR_IDX: u8 = 0x02;
const ATA_SR_ERR: u8 = 0x01;

const ATA_ER_BBK: u8 = 0x80;
const ATA_ER_UNC: u8 = 0x40;
const ATA_ER_MC: u8 = 0x20;
const ATA_ER_IDNF: u8 = 0x10;
const ATA_ER_MCR: u8 = 0x08;
const ATA_ER_ABRT: u8 = 0x04;
const ATA_ER_TK0NF: u8 = 0x02;
const ATA_ER_AMNF: u8 = 0x01;

const ATA_CMD_READ_PIO: u8 = 0x20;
const ATA_CMD_READ_PIO_EXT: u8 = 0x24;
const ATA_CMD_READ_DMA: u8 = 0xC8;
const ATA_CMD_READ_DMA_EXT: u8 = 0x25;
const ATA_CMD_WRITE_PIO: u8 = 0x30;
const ATA_CMD_WRITE_PIO_EXT: u8 = 0x34;
const ATA_CMD_WRITE_DMA: u8 = 0xCA;
const ATA_CMD_WRITE_DMA_EXT: u8 = 0x35;
const ATA_CMD_CACHE_FLUSH: u8 = 0xE7;
const ATA_CMD_CACHE_FLUSH_EXT: u8 = 0xEA;
const ATA_CMD_PACKET: u8 = 0xA0;
const ATA_CMD_IDENTIFY_PACKET: u8 = 0xA1;
const ATA_CMD_IDENTIFY: u8 = 0xEC;

const ATAPI_CMD_READ: u8 = 0xA8;
const ATAPI_CMD_EJECT: u8 = 0x1B;

const ATA_IDENT_DEVICETYPE: usize = 0;
const ATA_IDENT_CYLINDERS: usize = 2;
const ATA_IDENT_HEADS: usize = 6;
const ATA_IDENT_SECTORS: usize = 12;
const ATA_IDENT_SERIAL: usize = 20;
const ATA_IDENT_MODEL: usize = 54;
const ATA_IDENT_CAPABILITIES: usize = 98;
const ATA_IDENT_FIELDVALID: usize = 106;
const ATA_IDENT_MAX_LBA: usize = 120;
const ATA_IDENT_COMMANDSETS: usize = 164;
const ATA_IDENT_MAX_LBA_EXT: usize = 200;

const IDE_ATA: u8 = 0x00;
const IDE_ATAPI: u8 = 0x01;

const ATA_MASTER: u8 = 0x00;
const ATA_SLAVE: u8 = 0x01;

const ATA_REG_DATA: u8 = 0x00;
const ATA_REG_ERROR: u8 = 0x01;
const ATA_REG_FEATURES: u8 = 0x01;
const ATA_REG_SECCOUNT0: u8 = 0x02;
const ATA_REG_LBA0: u8 = 0x03;
const ATA_REG_LBA1: u8 = 0x04;
const ATA_REG_LBA2: u8 = 0x05;
const ATA_REG_HDDEVSEL: u8 = 0x06;
const ATA_REG_COMMAND: u8 = 0x07;
const ATA_REG_STATUS: u8 = 0x07;
const ATA_REG_SECCOUNT1: u8 = 0x08;
const ATA_REG_LBA3: u8 = 0x09;
const ATA_REG_LBA4: u8 = 0x0A;
const ATA_REG_LBA5: u8 = 0x0B;
const ATA_REG_CONTROL: u8 = 0x0C;
const ATA_REG_ALTSTATUS: u8 = 0x0C;
const ATA_REG_DEVADDRESS: u8 = 0x0D;

const ATA_PRIMARY: u8 = 0x00;
const ATA_SECONDARY: u8 = 0x01;

const ATA_READ: u8 = 0x00;
const ATA_WRITE: u8 = 0x01;

const MAX_SECTORS_PER_TRANSFER: usize = 128;

#[repr(C)]
struct IDEChannelRegisters {
    base: u16,
    ctrl: u16,
    bmide: u16,
    nien: u8,
}

static mut CHANNELS: [IDEChannelRegisters; 2] = [
    IDEChannelRegisters { base: 0, ctrl: 0, bmide: 0, nien: 0 },
    IDEChannelRegisters { base: 0, ctrl: 0, bmide: 0, nien: 0 },
];

static mut IDE_BUF: [u8; 512] = [0; 512];
static mut IDE_IRQ_INVOKED: u8 = 0;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IdeDevice {
    reserved: u8,
    channel: u8,
    drive: u8,
    device_type: u16,
    signature: u16,
    capabilities: u16,
    command_sets: u32,
    pub size: u64,
    pub model: [u8; 41],
}

pub static mut IDE_DEVICES: [IdeDevice; 4] = [
    IdeDevice { reserved: 0, channel: 0, drive: 0, device_type: 0, signature: 0, capabilities: 0, command_sets: 0, size: 0, model: [0; 41] },
    IdeDevice { reserved: 0, channel: 0, drive: 0, device_type: 0, signature: 0, capabilities: 0, command_sets: 0, size: 0, model: [0; 41] },
    IdeDevice { reserved: 0, channel: 0, drive: 0, device_type: 0, signature: 0, capabilities: 0, command_sets: 0, size: 0, model: [0; 41] },
    IdeDevice { reserved: 0, channel: 0, drive: 0, device_type: 0, signature: 0, capabilities: 0, command_sets: 0, size: 0, model: [0; 41] },
];

fn ide_write(channel: u8, reg: u8, data: u8) {
    unsafe {
        if reg > 0x07 && reg < 0x0C {
            outb!(CHANNELS[channel as usize].ctrl, 0x80 | CHANNELS[channel as usize].nien);
        }
        match reg {
            0x00..=0x07 => outb!(CHANNELS[channel as usize].base + (reg - 0x00) as u16, data),
            0x08..=0x0B => outb!(CHANNELS[channel as usize].base + (reg - 0x06) as u16, data),
            0x0C..=0x0D => outb!(CHANNELS[channel as usize].ctrl + (reg - 0x0A) as u16, data),
            0x0E..=0x15 => outb!(CHANNELS[channel as usize].bmide + (reg - 0x0E) as u16, data),
            _ => {}
        }
        if reg > 0x07 && reg < 0x0C {
            outb!(CHANNELS[channel as usize].ctrl, CHANNELS[channel as usize].nien);
        }
    }
}

fn ide_read(channel: u8, reg: u8) -> u8 {
    unsafe {
        if reg > 0x07 && reg < 0x0C {
            outb!(CHANNELS[channel as usize].ctrl, 0x80 | CHANNELS[channel as usize].nien);
        }
        let result = match reg {
            0x00..=0x07 => inb!(CHANNELS[channel as usize].base + (reg - 0x00) as u16),
            0x08..=0x0B => inb!(CHANNELS[channel as usize].base + (reg - 0x06) as u16),
            0x0C..=0x0D => inb!(CHANNELS[channel as usize].ctrl + (reg - 0x0A) as u16),
            0x0E..=0x15 => inb!(CHANNELS[channel as usize].bmide + (reg - 0x0E) as u16),
            _ => 0,
        };
        if reg > 0x07 && reg < 0x0C {
            outb!(CHANNELS[channel as usize].ctrl, CHANNELS[channel as usize].nien);
        }
        result
    }
}

fn ide_read_buffer(channel: u8, reg: u8, buffer: *mut u32, quads: u32) {
    unsafe {
        if reg > 0x07 && reg < 0x0C {
            outb!(CHANNELS[channel as usize].ctrl, 0x80 | CHANNELS[channel as usize].nien);
        }
        let port = match reg {
            0x00..=0x07 => CHANNELS[channel as usize].base + (reg - 0x00) as u16,
            0x08..=0x0B => CHANNELS[channel as usize].base + (reg - 0x06) as u16,
            0x0C..=0x0D => CHANNELS[channel as usize].ctrl + (reg - 0x0A) as u16,
            0x0E..=0x15 => CHANNELS[channel as usize].bmide + (reg - 0x0E) as u16,
            _ => return,
        };
        for i in 0..quads {
            *buffer.add(i as usize) = inl!(port);
        }
        if reg > 0x07 && reg < 0x0C {
            outb!(CHANNELS[channel as usize].ctrl, CHANNELS[channel as usize].nien);
        }
    }
}

fn ide_write_buffer(channel: u8, reg: u8, buffer: *const u32, quads: u32) {
    unsafe {
        if reg > 0x07 && reg < 0x0C {
            outb!(CHANNELS[channel as usize].ctrl, 0x80 | CHANNELS[channel as usize].nien);
        }
        let port = match reg {
            0x00..=0x07 => CHANNELS[channel as usize].base + (reg - 0x00) as u16,
            0x08..=0x0B => CHANNELS[channel as usize].base + (reg - 0x06) as u16,
            0x0C..=0x0D => CHANNELS[channel as usize].ctrl + (reg - 0x0A) as u16,
            0x0E..=0x15 => CHANNELS[channel as usize].bmide + (reg - 0x0E) as u16,
            _ => return,
        };
        for i in 0..quads {
            outl!(port, *buffer.add(i as usize));
        }
        if reg > 0x07 && reg < 0x0C {
            outb!(CHANNELS[channel as usize].ctrl, CHANNELS[channel as usize].nien);
        }
    }
}

fn ide_polling(channel: u8, advanced_check: bool) -> u8 {
    for _ in 0..4 {
        let _ = ide_read(channel, ATA_REG_ALTSTATUS);
    }

    let mut timeout = 100000;
    loop {
        let status = ide_read(channel, ATA_REG_STATUS);
        if (status & ATA_SR_BSY) == 0 {
            break;
        }
        timeout -= 1;
        if timeout == 0 {
            return 3;
        }
    }

    if advanced_check {
        let status = ide_read(channel, ATA_REG_STATUS);
        if (status & ATA_SR_ERR) != 0 {
            return 2;
        }
        if (status & ATA_SR_DF) != 0 {
            return 1;
        }
        if (status & ATA_SR_DRQ) == 0 {
            return 3;
        }
    }

    0
}

fn ide_wait_irq(channel: u8) -> u8 {
    unsafe {
        let mut timeout: usize = 1_000_000;
        while IDE_IRQ_INVOKED == 0 && timeout > 0 {
            timeout -= 1;
        }
        IDE_IRQ_INVOKED = 0;

        let status = ide_read(channel, ATA_REG_STATUS);
        if (status & ATA_SR_ERR) != 0 { return 2; }
        if (status & ATA_SR_DF) != 0 { return 1; }
        if (status & ATA_SR_DRQ) == 0 { return 3; }
        0
    }
}

pub fn ide_irq_handler() {
    unsafe {
        let _ = ide_read(ATA_PRIMARY, ATA_REG_STATUS);
        let _ = ide_read(ATA_SECONDARY, ATA_REG_STATUS);
        IDE_IRQ_INVOKED = 1;
    }
}

fn ide_print_error(drive: usize, mut err: u8) -> u8 {
    if err == 0 {
        return err;
    }

    println!("IDE:");
    if err == 1 {
        println!("- Device Fault");
        err = 19;
    } else if err == 2 {
        let st = ide_read(unsafe { IDE_DEVICES }[drive].channel, ATA_REG_ERROR);
        if st & ATA_ER_AMNF != 0 { println!("- No Address Mark Found"); err = 7; }
        if st & ATA_ER_TK0NF != 0 { println!("- No Media or Media Error"); err = 3; }
        if st & ATA_ER_ABRT != 0 { println!("- Command Aborted"); err = 20; }
        if st & ATA_ER_MCR != 0 { println!("- No Media or Media Error"); err = 3; }
        if st & ATA_ER_IDNF != 0 { println!("- ID mark not Found"); err = 21; }
        if st & ATA_ER_MC != 0 { println!("- No Media or Media Error"); err = 3; }
        if st & ATA_ER_UNC != 0 { println!("- Uncorrectable Data Error"); err = 22; }
        if st & ATA_ER_BBK != 0 { println!("- Bad Sectors"); err = 13; }
    } else if err == 3 {
        println!("- Reads Nothing");
        err = 23;
    } else if err == 4 {
        println!("- Write Protected");
        err = 8;
    }

    println!(
        "- [{} {}] {}",
        ["Primary", "Secondary"][unsafe { IDE_DEVICES }[drive].channel as usize],
        ["Master", "Slave"][unsafe { IDE_DEVICES }[drive].drive as usize],
        core::str::from_utf8(&unsafe { IDE_DEVICES }[drive].model).unwrap_or("Unknown")
    );

    err
}

pub fn ide_read_sectors(drive: usize, lba: u64, buffer: &mut Vec<u8>) -> u8 {
    unsafe {
        let dev = &IDE_DEVICES[drive];
        if dev.reserved == 0 { return 1; }
        let channel = dev.channel;
        let drive_bit = dev.drive;
        let total_sectors = buffer.len() / 512;
        buffer.resize(total_sectors * 512, 0);
        
        let mut sectors_read = 0;
        
        while sectors_read < total_sectors {
            let sectors_to_read = core::cmp::min(MAX_SECTORS_PER_TRANSFER, total_sectors - sectors_read);
            let current_lba = lba + sectors_read as u64;
            let use_lba48 = current_lba >= 0x10000000 || sectors_to_read > 256;
            
            while (ide_read(channel, ATA_REG_STATUS) & ATA_SR_BSY) != 0 {}
            
            if use_lba48 {
                ide_write(channel, ATA_REG_HDDEVSEL, 0x40 | ((drive_bit as u8) << 4));
                ide_write(channel, ATA_REG_SECCOUNT1, ((sectors_to_read >> 8) & 0xFF) as u8);
                ide_write(channel, ATA_REG_LBA3, ((current_lba >> 24) & 0xFF) as u8);
                ide_write(channel, ATA_REG_LBA4, ((current_lba >> 32) & 0xFF) as u8);
                ide_write(channel, ATA_REG_LBA5, ((current_lba >> 40) & 0xFF) as u8);
                ide_write(channel, ATA_REG_SECCOUNT0, (sectors_to_read & 0xFF) as u8);
                ide_write(channel, ATA_REG_LBA0, (current_lba & 0xFF) as u8);
                ide_write(channel, ATA_REG_LBA1, ((current_lba >> 8) & 0xFF) as u8);
                ide_write(channel, ATA_REG_LBA2, ((current_lba >> 16) & 0xFF) as u8);
                ide_write(channel, ATA_REG_COMMAND, ATA_CMD_READ_PIO_EXT);
            } else {
                ide_write(channel, ATA_REG_HDDEVSEL, 
                    0xE0 | ((drive_bit as u8) << 4) | ((current_lba >> 24) & 0x0F) as u8);
                ide_write(channel, ATA_REG_SECCOUNT0, sectors_to_read as u8);
                ide_write(channel, ATA_REG_LBA0, (current_lba & 0xFF) as u8);
                ide_write(channel, ATA_REG_LBA1, ((current_lba >> 8) & 0xFF) as u8);
                ide_write(channel, ATA_REG_LBA2, ((current_lba >> 16) & 0xFF) as u8);
                ide_write(channel, ATA_REG_COMMAND, ATA_CMD_READ_PIO);
            }
            
            for s in 0..sectors_to_read {
                let err = ide_wait_irq(channel);
                if err != 0 { return ide_print_error(drive, err); }
                
                let offset = (sectors_read + s) * 512;
                ide_read_buffer(channel, ATA_REG_DATA, 
                    buffer.as_mut_ptr().add(offset).cast::<u32>(), 128);
            }
            
            sectors_read += sectors_to_read;
        }
        0
    }
}

pub fn ide_write_sectors(drive: usize, lba: u64, data: &[u8]) -> u8 {
    unsafe {
        let dev = &IDE_DEVICES[drive];
        if dev.reserved == 0 { return 1; }
        let channel = dev.channel;
        let drive_bit = dev.drive;
        let data_size = data.len();
        let total_sectors = (data_size + 511) / 512;
        
        let mut sectors_written = 0;
        
        while sectors_written < total_sectors {
            let sectors_to_write = core::cmp::min(MAX_SECTORS_PER_TRANSFER, total_sectors - sectors_written);
            let current_lba = lba + sectors_written as u64;
            let use_lba48 = current_lba >= 0x10000000 || sectors_to_write > 256;
            
            while (ide_read(channel, ATA_REG_STATUS) & ATA_SR_BSY) != 0 {}
            
            if use_lba48 {
                ide_write(channel, ATA_REG_HDDEVSEL, 0x40 | ((drive_bit as u8) << 4));
                for _ in 0..4 {
                    ide_read(channel, ATA_REG_ALTSTATUS);
                }
                ide_write(channel, ATA_REG_SECCOUNT1, ((sectors_to_write >> 8) & 0xFF) as u8);
                ide_write(channel, ATA_REG_LBA3, ((current_lba >> 24) & 0xFF) as u8);
                ide_write(channel, ATA_REG_LBA4, ((current_lba >> 32) & 0xFF) as u8);
                ide_write(channel, ATA_REG_LBA5, ((current_lba >> 40) & 0xFF) as u8);
                ide_write(channel, ATA_REG_SECCOUNT0, (sectors_to_write & 0xFF) as u8);
                ide_write(channel, ATA_REG_LBA0, (current_lba & 0xFF) as u8);
                ide_write(channel, ATA_REG_LBA1, ((current_lba >> 8) & 0xFF) as u8);
                ide_write(channel, ATA_REG_LBA2, ((current_lba >> 16) & 0xFF) as u8);
                ide_write(channel, ATA_REG_COMMAND, ATA_CMD_WRITE_PIO_EXT);
            } else {
                ide_write(channel, ATA_REG_HDDEVSEL, 
                    0xE0 | ((drive_bit as u8) << 4) | (((current_lba >> 24) & 0x0F) as u8));
                for _ in 0..4 {
                    ide_read(channel, ATA_REG_ALTSTATUS);
                }
                ide_write(channel, ATA_REG_SECCOUNT0, sectors_to_write as u8);
                ide_write(channel, ATA_REG_LBA0, (current_lba & 0xFF) as u8);
                ide_write(channel, ATA_REG_LBA1, ((current_lba >> 8) & 0xFF) as u8);
                ide_write(channel, ATA_REG_LBA2, ((current_lba >> 16) & 0xFF) as u8);
                ide_write(channel, ATA_REG_COMMAND, ATA_CMD_WRITE_PIO);
            }
            
            for s in 0..sectors_to_write {
                let err = ide_polling(channel, true);
                if err != 0 { return ide_print_error(drive, err); }
                
                let offset = (sectors_written + s) * 512;
                let bytes_left = data_size.saturating_sub(offset);
                let bytes_to_write = core::cmp::min(512, bytes_left);
                
                if bytes_to_write >= 512 {
                    ide_write_buffer(channel, ATA_REG_DATA,
                        data.as_ptr().add(offset).cast::<u32>(), 128);
                } else {
                    let mut padded = [0u8; 512];
                    padded[..bytes_to_write].copy_from_slice(&data[offset..offset + bytes_to_write]);
                    ide_write_buffer(channel, ATA_REG_DATA,
                        padded.as_ptr().cast::<u32>(), 128);
                }
            }
            
            sectors_written += sectors_to_write;
        }
        
        if lba >= 0x10000000 {
            ide_write(channel, ATA_REG_COMMAND, ATA_CMD_CACHE_FLUSH_EXT);
        } else {
            ide_write(channel, ATA_REG_COMMAND, ATA_CMD_CACHE_FLUSH);
        }
        ide_polling(channel, false);
        0
    }
}

pub fn ide_init(bar0: u8, bar1: u8, bar2: u8, bar3: u8, bar4: u8) {
    let mut count: usize = 0;

    unsafe {
        CHANNELS[ATA_PRIMARY as usize].base =
            (((bar0 as u32) & 0xFFFF_FFFC) + if bar0 == 0 { 0x1F0 } else { 0 }) as u16;
        CHANNELS[ATA_PRIMARY as usize].ctrl =
            (((bar1 as u32) & 0xFFFF_FFFC) + if bar1 == 0 { 0x3F6 } else { 0 }) as u16;

        CHANNELS[ATA_SECONDARY as usize].base =
            (((bar2 as u32) & 0xFFFF_FFFC) + if bar2 == 0 { 0x170 } else { 0 }) as u16;
        CHANNELS[ATA_SECONDARY as usize].ctrl =
            (((bar3 as u32) & 0xFFFF_FFFC) + if bar3 == 0 { 0x376 } else { 0 }) as u16;

        let bm = ((bar4 as u32) & 0xFFFF_FFFC) as u16;
        CHANNELS[ATA_PRIMARY as usize].bmide = bm;
        CHANNELS[ATA_SECONDARY as usize].bmide = bm.wrapping_add(8);

        CHANNELS[ATA_PRIMARY as usize].nien = 0;
        CHANNELS[ATA_SECONDARY as usize].nien = 0;

        for j in 0..2 {
            outb!(CHANNELS[j].ctrl, 0x02);
        }

        for j in 0..2 {
            for k in 0..2 {
                let drive_index = j * 2 + k;
                IDE_DEVICES[drive_index].reserved = 0;
                IDE_DEVICES[drive_index].channel = j as u8;
                IDE_DEVICES[drive_index].drive = k as u8;

                ide_write(j as u8, ATA_REG_HDDEVSEL, 0xA0 | ((k as u8) << 4));
                for _ in 0..4 { let _ = ide_read(j as u8, ATA_REG_STATUS); }

                ide_write(j as u8, ATA_REG_COMMAND, ATA_CMD_IDENTIFY);
                
                let mut timeout = 100000;
                loop {
                    let status = ide_read(j as u8, ATA_REG_STATUS);
                    if status == 0 { break; }
                    if (status & ATA_SR_ERR) != 0 { break; }
                    if (status & ATA_SR_BSY) == 0 && (status & ATA_SR_DRQ) != 0 {
                        let mut buf = IDE_BUF;
                        ide_read_buffer(j as u8, ATA_REG_DATA, buf.as_mut_ptr().cast::<u32>(), 128);

                        let model_offset = ATA_IDENT_MODEL;
                        for m in 0..40 {
                            IDE_DEVICES[drive_index].model[m] = buf[model_offset + m];
                        }

                        IDE_DEVICES[drive_index].model[40] = 0;
                        IDE_DEVICES[drive_index].reserved = 1;
                        IDE_DEVICES[drive_index].device_type = IDE_ATA as u16;

                        let commands_sets = u16::from_le_bytes([
                            buf[ATA_IDENT_COMMANDSETS],
                            buf[ATA_IDENT_COMMANDSETS + 1]
                        ]);

                        let lba48 = (commands_sets & (1 << 10)) != 0;

                        if lba48 {
                            IDE_DEVICES[drive_index].size = u64::from_le_bytes([
                                buf[ATA_IDENT_MAX_LBA_EXT],
                                buf[ATA_IDENT_MAX_LBA_EXT + 1],
                                buf[ATA_IDENT_MAX_LBA_EXT + 2],
                                buf[ATA_IDENT_MAX_LBA_EXT + 3],
                                buf[ATA_IDENT_MAX_LBA_EXT + 4],
                                buf[ATA_IDENT_MAX_LBA_EXT + 5],
                                0,
                                0,
                            ]);
                            println!("Device {}: LBA48, Size: {} sectors", 
                                   drive_index, IDE_DEVICES[drive_index].size);
                        } else {
                            IDE_DEVICES[drive_index].size = u64::from_le_bytes([
                                buf[ATA_IDENT_MAX_LBA],
                                buf[ATA_IDENT_MAX_LBA + 1],
                                buf[ATA_IDENT_MAX_LBA + 2],
                                buf[ATA_IDENT_MAX_LBA + 3],
                                0,
                                0,
                                0,
                                0,
                            ]);
                            println!("Device {}: LBA28, Size: {} sectors", 
                                   drive_index, IDE_DEVICES[drive_index].size);
                        }
                        
                        count += 1;
                        break;
                    }
                    
                    timeout -= 1;
                    if timeout == 0 { break; }
                }
            }
        }

        for j in 0..2 {
            outb!(CHANNELS[j].ctrl, 0x00);
        }
    }

    if count == 0 {
        println!("IDE: No devices found");
    } else {
        println!("IDE: devices detected: {}", count);
    }
}