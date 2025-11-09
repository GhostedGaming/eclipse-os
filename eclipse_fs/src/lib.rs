#![no_std]
extern crate alloc;

use eclipse_ide::{ide_read_sectors, ide_write_sectors};
use eclipse_framebuffer::println;
use super_block::SuperBlock;
use alloc::vec::Vec;

mod super_block;

pub fn write_eclipse_fs(drive: u8) {
    let super_block = SuperBlock::new(drive);
    
    println!("SuperBlock size: {} bytes", core::mem::size_of::<SuperBlock>());
    println!("This will write {} sectors", (core::mem::size_of::<SuperBlock>() + 511) / 512);
    println!("Writing eclipse superblock");
    
    if ide_write_sectors(drive as usize, 1, &super_block) != 0 {
        println!("Failed to write superblock");
        return;
    }
    
    println!("Superblock wrote to disk");
    
    let mut buf: Vec<u8> = alloc::vec![0u8; 512];
    
    if ide_read_sectors(drive as usize, 1, &mut buf) != 0 {
        println!("Failed to read superblock back");
        return;
    }
    
    println!("Superblock: {:?}", buf);
}