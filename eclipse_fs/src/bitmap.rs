use alloc::{vec::Vec, vec};
use eclipse_ide::{ide_read_sectors, ide_write_sectors};
use super::SuperBlock;

#[derive(Debug)]
pub enum BitmapError {
    FailedToReadBitmap,
    FailedToWriteBitmap,
    InvalidDrive,
    InvalidBlock,
}

pub struct BlockBitmap {
    bits: Vec<u8>,
    total_blocks: u64,
}

impl BlockBitmap {
    pub fn new(super_block: &SuperBlock) -> Self {
        let total_blocks = super_block.blocks();
        let bitmap_bytes = ((total_blocks + 7) / 8) as usize;
        let mut bits = vec![0u8; bitmap_bytes];

        for block in 0..super_block.data_region_start {
            Self::set_bit(&mut bits, block as usize);
        }

        Self {
            bits,
            total_blocks,
        }
    }

    pub fn from_disk(drive: usize, super_block: &SuperBlock) -> Result<Self, BitmapError> {
        if drive >= 4 {
            return Err(BitmapError::InvalidDrive);
        }

        let total_blocks = super_block.blocks();
        let bitmap_bytes = ((total_blocks + 7) / 8) as usize;
        let sectors_per_block = (super_block.block_size / 512) as u64;
        let start_sector = super_block.block_bitmap_start * sectors_per_block;
        
        let sectors_needed = ((bitmap_bytes + 511) / 512) as u64;
        let mut bits = vec![0u8; sectors_needed as usize * 512];

        for sector_offset in 0..sectors_needed {
            let sector = start_sector + sector_offset;
            let offset = (sector_offset as usize) * 512;
            
            if ide_read_sectors(drive, sector, &mut bits[offset..offset + 512]) != 0 {
                return Err(BitmapError::FailedToReadBitmap);
            }
        }

        bits.truncate(bitmap_bytes);

        Ok(Self {
            bits,
            total_blocks,
        })
    }

    pub fn write_to_disk(&self, drive: usize, super_block: &SuperBlock) -> Result<(), BitmapError> {
        if drive >= 4 {
            return Err(BitmapError::InvalidDrive);
        }

        let sectors_per_block = (super_block.block_size / 512) as u64;
        let start_sector = super_block.block_bitmap_start * sectors_per_block;
        
        let mut padded_bits = self.bits.clone();
        let remainder = padded_bits.len() % 512;
        if remainder != 0 {
            padded_bits.resize(padded_bits.len() + (512 - remainder), 0);
        }

        for (i, chunk) in padded_bits.chunks(512).enumerate() {
            let sector = start_sector + i as u64;
            let mut buffer = chunk.to_vec();
            
            if ide_write_sectors(drive, sector, &mut buffer) != 0 {
                return Err(BitmapError::FailedToWriteBitmap);
            }
        }

        Ok(())
    }

    pub fn allocate_block(&mut self) -> Result<u64, BitmapError> {
        for block in 0..self.total_blocks as usize {
            if !self.is_allocated(block) {
                Self::set_bit(&mut self.bits, block);
                return Ok(block as u64);
            }
        }
        Err(BitmapError::InvalidBlock)
    }

    pub fn allocate_specified_block(&mut self, block: u64) -> Result<(), BitmapError> {
        if block >= self.total_blocks {
            return Err(BitmapError::InvalidBlock);
        }
        if self.is_allocated(block as usize) {
            return Ok(());
        }
        Self::set_bit(&mut self.bits, block as usize);
        Ok(())
    }

    pub fn free_block(&mut self, block: u64) -> Result<(), BitmapError> {
        if block >= self.total_blocks {
            return Err(BitmapError::InvalidBlock);
        }
        Self::clear_bit(&mut self.bits, block as usize);
        Ok(())
    }

    pub fn is_allocated(&self, block: usize) -> bool {
        if block >= self.total_blocks as usize {
            return false;
        }
        let byte_idx = block / 8;
        let bit_idx = block % 8;
        (self.bits[byte_idx] & (1 << bit_idx)) != 0
    }

    fn set_bit(bits: &mut [u8], block: usize) {
        let byte_idx = block / 8;
        let bit_idx = block % 8;
        if byte_idx < bits.len() {
            bits[byte_idx] |= 1 << bit_idx;
        }
    }

    fn clear_bit(bits: &mut [u8], block: usize) {
        let byte_idx = block / 8;
        let bit_idx = block % 8;
        if byte_idx < bits.len() {
            bits[byte_idx] &= !(1 << bit_idx);
        }
    }

    pub fn free_blocks(&self) -> u64 {
        let mut count = 0;
        for block in 0..self.total_blocks as usize {
            if !self.is_allocated(block) {
                count += 1;
            }
        }
        count
    }

    pub fn used_blocks(&self) -> u64 {
        self.total_blocks - self.free_blocks()
    }
}