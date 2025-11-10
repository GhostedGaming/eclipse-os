#![no_std]
extern crate alloc;
use eclipse_ide::{ide_read_sectors, ide_write_sectors};
use eclipse_framebuffer::println;
use alloc::vec::Vec;

// Exported functions and modules
pub use super_block::SuperBlock;
pub use block_io::{read_block, write_block, BlockError};

mod super_block;
mod block_io;

fn zero_sector(drive: usize, start_block: u64, num_blocks: u64, block_size_bytes: u64) -> bool {
    let sector_size: u64 = 512;
    let sectors_per_block = block_size_bytes / sector_size;
    
    let start_sector = start_block * sectors_per_block;
    let total_sectors = num_blocks * sectors_per_block;
    
    let zero_sector = [0u8; 512];

    for i in 0..total_sectors {
        let sector_to_write = start_sector + i;

        if ide_write_sectors(drive, sector_to_write as u32, &zero_sector) != 0 {
            println!("Writing zeros failed at: {}", sector_to_write);
            return false;
        }
    }

    true
}


pub fn write_eclipse_fs(drive: u8) {
    let drive_usize = drive as usize;
    let super_block = SuperBlock::new(drive);
    
    println!("SuperBlock Layout: {}", super_block);

    let sb_bytes_512 = super_block.to_bytes();
    
    if ide_write_sectors(drive_usize, 1, &sb_bytes_512) != 0 {
        println!("Failed to write superblock");
        return;
    }
    println!("Superblock written to disk.");

    println!("Initializing Inode: {}", super_block.inode_table_blocks);
    if !zero_sector(
        drive_usize,
        super_block.inode_table_start,
        super_block.inode_table_blocks,
        super_block.block_size
    ) {
        return;
    }
    println!("Inode Table initialized.");


    println!("Initializing Block Bitmap region ({} blocks)", super_block.block_bitmap_blocks);
    if !zero_sector(
        drive_usize,
        super_block.block_bitmap_start,
        super_block.block_bitmap_blocks,
        super_block.block_size
    ) {
        return;
    }
    println!("Block Bitmap initialized.");
    
    println!("Initializing Reserved region ({} blocks)", super_block.reserved_blocks);
    if !zero_sector(
        drive_usize,
        super_block.reserved_start,
        super_block.reserved_blocks,
        super_block.block_size
    ) {
        return;
    }
    println!("Reserved region initialized.");
    
    println!("Verifying superblock read-back...");
    let mut buf: Vec<u8> = alloc::vec![0u8; 512];
    if ide_read_sectors(drive_usize, 1, &mut buf) != 0 {
        println!("Failed to read superblock");
        return;
    }
    
    match SuperBlock::from_bytes(&buf) {
        Ok(sb) => println!("Verification successful: {}", sb),
        Err(e) => println!("Verification failed: {}", e),
    }

    println!("Filesystem initialization complete.");
}