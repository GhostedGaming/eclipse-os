use alloc::{vec, vec::Vec};
use eclipse_framebuffer::{print, println};
use eclipse_ide::{ide_read_sectors, ide_write_sectors};
use crate::super_block::SuperBlock;

#[derive(Debug)]
pub enum BlockError {
    OutOfBounds,
    ReadFailed,
    WriteFailed,
    InvalidBlockSize,
    InvalidDrive,
}

pub fn read_block(drive: usize, super_block: &SuperBlock, block: u64) -> Result<Vec<u8>, BlockError> {
    let block_count = super_block.blocks;
    let mut block_size = super_block.block_size;

    if drive > 4 {
        return Err(BlockError::InvalidDrive);
    }
    
    if block >= block_count {
        println!("Block {} is greater than or equal to Block Count: {}", block, block_count);
        return Err(BlockError::OutOfBounds);
    }
    
    if block_size % 512 != 0 {
        block_size = ((block_size + 511) / 512) * 512;
        println!("Block size padded to {}", block_size);
    }
    
    let mut buffer = vec![0u8; block_size as usize];
    let sectors_per_block = block_size / 512;
    let lba = block * sectors_per_block as u64;
    println!("Read lba: {}", lba);
    
    if ide_read_sectors(drive, lba, &mut buffer) != 0 {
        return Err(BlockError::ReadFailed);
    }
    
    buffer.truncate(super_block.block_size as usize);
    Ok(buffer)
}

pub fn write_block(drive: usize, super_block: &SuperBlock, block: u64, data: &[u8]) -> Result<(), BlockError> {
    let block_count = super_block.blocks;
    let mut block_size = super_block.block_size;

    if drive > 4 {
        return Err(BlockError::InvalidDrive);
    }
    
    if block >= block_count {
        println!("Block {} is greater than or equal to Block Count: {}", block, block_count);
        return Err(BlockError::OutOfBounds);
    }
    
    if data.len() > block_size as usize {
        println!("Data size {} is larger than block size {}", data.len(), block_size);
        return Err(BlockError::InvalidBlockSize);
    }
    
    let mut buffer: Vec<u8> = data.to_vec();
    if data.len() < block_size as usize {
        println!("Padding data from {} to {} bytes", data.len(), block_size);
        buffer.resize(block_size as usize, 0);
    }
    
    if block_size % 512 != 0 {
        let padded_size = ((block_size + 511) / 512) * 512;
        println!("Block size padded from {} to {}", block_size, padded_size);
        buffer.resize(padded_size as usize, 0);
        block_size = padded_size;
    }
    
    let sectors_per_block = block_size / 512;
    let lba = block * sectors_per_block as u64;

    println!("Write lba: {}", lba);

    println!("Printing first 126 bytes of the buffer");
    for i in 0..126 {
        print!("0x{:02X?} ", &buffer[i]);
    }
    
    if ide_write_sectors(drive, lba, &buffer) != 0 {
        return Err(BlockError::WriteFailed);
    }
    
    Ok(())
}