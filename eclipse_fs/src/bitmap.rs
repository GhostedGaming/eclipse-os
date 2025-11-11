use alloc::{vec::Vec, vec};
use eclipse_framebuffer::println;
use eclipse_ide::{IDE_DEVICES, ide_read_sectors, ide_write_sectors};
use super::SuperBlock;

#[derive(Debug)]
pub enum BitmapError {
    FailedToWriteBit,
    FailedToReadBitmap,
    FailedToCreateBitmap,
    FailedToWriteBitmap,
    InvalidDrive,
}

// Reads the whole drive and creates a bitmap and updates the old one
fn return_new_bitmap(drive: usize, super_block: &SuperBlock) -> Result<Vec<u8>, BitmapError> {
    println!("Creating new bitmap");
    unsafe {
        if drive >= 4 {
            return Err(BitmapError::InvalidDrive);
        }
        let dev = IDE_DEVICES[drive];
        let block_size = super_block.block_size as usize;
        let dev_size = dev.size;
        let mut bitmap = Vec::new();
        const SECTOR_SIZE: usize = 512;
        let sectors_per_block = (block_size + SECTOR_SIZE - 1) / SECTOR_SIZE;
        
        for block_idx in 0..(dev_size as usize / sectors_per_block) {
            let mut buffer = vec![0u8; block_size];
            for sector_offset in 0..sectors_per_block {
                let sector = (block_idx * sectors_per_block + sector_offset) as u64;
                if sector >= dev_size {
                    break;
                }
                let mut sector_buffer = vec![0u8; SECTOR_SIZE];
                if ide_read_sectors(drive, sector, &mut sector_buffer) != 0 {
                    return Err(BitmapError::FailedToCreateBitmap);
                }
                let start = sector_offset * SECTOR_SIZE;
                let end = (start + SECTOR_SIZE).min(block_size);
                buffer[start..end].copy_from_slice(&sector_buffer[0..(end - start)]);
            }
            
            let mut is_used = false;
            for i in 0..block_size {
                if buffer[i] > 0 {
                    is_used = true;
                    println!("{}", i);
                    break;
                }
            }
            bitmap.push(if is_used { 1 } else { 0 });
        }
        Ok(bitmap)
    }
}

pub fn write_bitmap(drive: usize, super_block: &SuperBlock) -> Result<(), BitmapError> {
    println!("Attempting to write bitmap");
    if drive >= 4 {
        return Err(BitmapError::InvalidDrive);
    }
    
    let mut bitmap = return_new_bitmap(drive, super_block)?;
    
    const SECTOR_SIZE: usize = 512;
    let remainder = bitmap.len() % SECTOR_SIZE;
    if remainder != 0 {
        let padding_needed = SECTOR_SIZE - remainder;
        bitmap.resize(bitmap.len() + padding_needed, 0);
    }
    
    let block_size = super_block.block_size;
    let sectors_per_block = block_size / 512;
    let mut lba = super_block.block_bitmap_start * sectors_per_block;
    
    for chunk in bitmap.chunks(SECTOR_SIZE) {
        let mut sector_buffer = chunk.to_vec();
        if ide_write_sectors(drive, lba, &mut sector_buffer) != 0 {
            return Err(BitmapError::FailedToWriteBit);
        }
        println!("LBA: {}", lba);
        lba += 1;
    }
    
    Ok(())
}