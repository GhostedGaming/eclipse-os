#![no_std]
extern crate alloc;

use eclipse_ide::{ide_read_sectors, ide_write_sectors};
use eclipse_framebuffer::println;
use alloc::vec;

pub use super_block::SuperBlock;
pub use block_io::{read_block, write_block, BlockError};
pub use bitmap::{BlockBitmap, BitmapError};
pub use inodes::{InodeManager, Inode};

mod super_block;
mod block_io;
mod bitmap;
pub mod inodes;
pub mod file_ops;
pub mod directory;

pub trait StorageDriver {
    fn read_sector(&self, lba: u64, buffer: &mut [u8]) -> bool;
    fn write_sector(&self, lba: u64, data: &[u8]) -> bool;
}

pub struct IdeDriver {
    pub drive: usize,
}

impl StorageDriver for IdeDriver {
    fn read_sector(&self, lba: u64, buffer: &mut [u8]) -> bool {
        ide_read_sectors(self.drive, lba, buffer) == 0
    }

    fn write_sector(&self, lba: u64, data: &[u8]) -> bool {
        ide_write_sectors(self.drive, lba, data) == 0
    }
}

fn zero_sector(drive: usize, start_block: u64, num_blocks: u64, block_size_bytes: u64) -> bool {
    let sector_size: u64 = 512;
    let sectors_per_block = block_size_bytes / sector_size;
    let start_sector = start_block * sectors_per_block;
    let total_sectors = num_blocks * sectors_per_block;
    let zero_sector = [0u8; 512];
    
    for i in 0..total_sectors {
        let sector_to_write = start_sector + i;
        if ide_write_sectors(drive, sector_to_write, &zero_sector) != 0 {
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
    let mut buf = vec![0u8; 512];
    if ide_read_sectors(drive_usize, 1, &mut buf) != 0 {
        println!("Failed to read superblock");
        return;
    }
    
    match SuperBlock::from_bytes(&buf) {
        Ok(sb) => println!("Verification successful: {}", sb),
        Err(e) => println!("Verification failed: {}", e),
    }
    
    println!("Creating and writing bitmap...");
    let bitmap = BlockBitmap::new(&super_block);
    match bitmap.write_to_disk(drive_usize, &super_block) {
        Ok(()) => {
            println!("Bitmap written successfully");
            println!("Free blocks: {}", bitmap.free_blocks());
            println!("Used blocks: {}", bitmap.used_blocks());
        },
        Err(e) => println!("Bitmap write error: {:?}", e),
    }
    
    println!("Filesystem initialization complete.");
}