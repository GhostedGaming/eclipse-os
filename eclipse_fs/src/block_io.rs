use alloc::vec;
use eclipse_framebuffer::println;
use ide::{ide_read_sectors, ide_write_sectors};
use ahci::{HbaPort, ahci_read, ahci_write};
use crate::super_block::SuperBlock;
use crate::bitmap::{BlockBitmap, BitmapError};

#[derive(Debug)]
pub enum BlockError {
    OutOfBounds,
    ReadFailed,
    WriteFailed,
    InvalidBlockSize,
    InvalidDrive,
    BlockNotAllocated,
    BitmapError(BitmapError),
}

impl From<BitmapError> for BlockError {
    fn from(err: BitmapError) -> Self {
        BlockError::BitmapError(err)
    }
}

pub fn read_block(
    drive: usize,
    super_block: &SuperBlock,
    bitmap: &BlockBitmap,
    block: u64,
) -> Result<alloc::vec::Vec<u8>, BlockError> {
    let block_count = super_block.blocks;
    let mut block_size = super_block.block_size;
    
    if drive >= 4 {
        return Err(BlockError::InvalidDrive);
    }
    
    if block >= block_count {
        println!("Block {} is greater than or equal to Block Count: {}", block, block_count);
        return Err(BlockError::OutOfBounds);
    }
    
    if !bitmap.is_allocated(block as usize) {
        println!("Warning: Reading unallocated block {}", block);
    }
    
    if block_size % 512 != 0 {
        block_size = ((block_size + 511) / 512) * 512;
        println!("Block size padded to {}", block_size);
    }
    
    let mut buffer = vec![0u8; block_size as usize];
    let sectors_per_block = block_size / 512;
    let lba = block * sectors_per_block;
    
    println!("Read lba: {}", lba);
    
    if ide_read_sectors(drive, lba, &mut buffer) != 0 {
        return Err(BlockError::ReadFailed);
    }
    
    buffer.truncate(super_block.block_size as usize);
    Ok(buffer)
}

pub fn write_block(
    drive: usize,
    super_block: &SuperBlock,
    bitmap: &mut BlockBitmap,
    block: u64,
    data: &[u8],
) -> Result<(), BlockError> {
    let block_count = super_block.blocks;
    let mut block_size = super_block.block_size;
    
    if drive >= 4 {
        return Err(BlockError::InvalidDrive);
    }
    
    if block >= block_count {
        println!("Block {} is greater than or equal to Block Count: {}", block, block_count);
        return Err(BlockError::OutOfBounds);
    }
    
    if block < super_block.data_region_start {
        println!("Warning: Writing to system block {}", block);
    }
    
    if !bitmap.is_allocated(block as usize) {
        bitmap.allocate_specified_block(block)?;
        println!("Allocated block {} in bitmap", block);
    }
    
    if data.len() > block_size as usize {
        println!("Data size {} is larger than block size {}", data.len(), block_size);
        return Err(BlockError::InvalidBlockSize);
    }
    
    let mut buffer = data.to_vec();
    
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
    let lba = block * sectors_per_block;
    
    println!("Write lba: {}", lba);
    
    if ide_write_sectors(drive, lba, &buffer) != 0 {
        return Err(BlockError::WriteFailed);
    }
    
    Ok(())
}

pub fn read_block_ahci(
    port: &HbaPort,
    super_block: &SuperBlock,
    bitmap: &BlockBitmap,
    block: u64,
) -> Result<alloc::vec::Vec<u8>, BlockError> {
    let block_count = super_block.blocks;
    let mut block_size = super_block.block_size;
    
    if block >= block_count {
        println!("Block {} is greater than or equal to Block Count: {}", block, block_count);
        return Err(BlockError::OutOfBounds);
    }
    
    if !bitmap.is_allocated(block as usize) {
        println!("Warning: Reading unallocated block {}", block);
    }
    
    if block_size % 512 != 0 {
        block_size = ((block_size + 511) / 512) * 512;
        println!("Block size padded to {}", block_size);
    }
    
    let mut buffer = vec![0u8; block_size as usize];
    let sectors_per_block = block_size / 512;
    let lba = block * sectors_per_block;
    
    println!("Read lba: {}", lba);
    
    if !ahci_read(port, lba, sectors_per_block as u32, buffer.as_mut_ptr()) {
        return Err(BlockError::ReadFailed);
    }
    
    buffer.truncate(super_block.block_size as usize);
    Ok(buffer)
}

pub fn write_block_ahci(
    port: &HbaPort,
    super_block: &SuperBlock,
    bitmap: &mut BlockBitmap,
    block: u64,
    data: &[u8],
) -> Result<(), BlockError> {
    let block_count = super_block.blocks;
    let mut block_size = super_block.block_size;
    
    if block >= block_count {
        println!("Block {} is greater than or equal to Block Count: {}", block, block_count);
        return Err(BlockError::OutOfBounds);
    }
    
    if block < super_block.data_region_start {
        println!("Warning: Writing to system block {}", block);
    }
    
    if !bitmap.is_allocated(block as usize) {
        bitmap.allocate_specified_block(block)?;
        println!("Allocated block {} in bitmap", block);
    }
    
    if data.len() > block_size as usize {
        println!("Data size {} is larger than block size {}", data.len(), block_size);
        return Err(BlockError::InvalidBlockSize);
    }
    
    let mut buffer = data.to_vec();
    
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
    let lba = block * sectors_per_block;
    
    println!("Write lba: {}", lba);
    
    if !ahci_write(port, lba, sectors_per_block as u32, buffer.as_ptr()) {
        return Err(BlockError::WriteFailed);
    }
    
    Ok(())
}