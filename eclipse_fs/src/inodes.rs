use core::fmt;

use alloc::vec::Vec;

use crate::super_block::SuperBlock;
use crate::bitmap::{BlockBitmap, BitmapError};
use crate::block_io::{read_block, write_block, BlockError};
use eclipse_framebuffer::println;

#[derive(Debug)]
pub enum InodeError {
    OutOfBounds,
    ReadFailed,
    WriteFailed,
    InvalidInode,
    BitmapError(BitmapError),
    BlockError(BlockError),
}

impl From<BitmapError> for InodeError {
    fn from(err: BitmapError) -> Self {
        InodeError::BitmapError(err)
    }
}

impl From<BlockError> for InodeError {
    fn from(err: BlockError) -> Self {
        InodeError::BlockError(err)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Inode {
    pub size: u64,
    pub direct_blocks: [u64; 12],
    pub indirect_block: u64,
    pub double_indirect_block: u64,
}

impl Inode {
    pub fn new() -> Self {
        Inode {
            size: 0,
            direct_blocks: [0; 12],
            indirect_block: 0,
            double_indirect_block: 0,
        }
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let ptr = self as *const Inode as *const u8;
        unsafe {
            Vec::from(core::slice::from_raw_parts(ptr, core::mem::size_of::<Inode>()))
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, InodeError> {
        if bytes.len() < core::mem::size_of::<Inode>() {
            return Err(InodeError::ReadFailed);
        }
        unsafe {
            let mut inode = Inode::new();
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), &mut inode as *mut _ as *mut u8, core::mem::size_of::<Inode>());
            Ok(inode)
        }
    }
}

#[derive(Clone)]
pub struct InodeTable {
    pub inodes: Vec<Inode>,
}

impl InodeTable {
    pub fn new(capacity: usize) -> Self {
        InodeTable {
            inodes: Vec::with_capacity(capacity),
        }
    }

    pub fn from_disk(drive: usize, super_block: &SuperBlock) -> Result<Self, InodeError> {
        let mut inodes = Vec::new();
        let inode_size = core::mem::size_of::<Inode>() as u64;
        let inodes_per_block = super_block.block_size / inode_size;
        
        for i in 0..super_block.inode_table_blocks {
            let block = super_block.inode_table_start + i;
            let block_data = read_block(drive, super_block, &BlockBitmap::new(super_block), block)?;
            
            for j in 0..inodes_per_block {
                let offset = (j * inode_size) as usize;
                if offset + core::mem::size_of::<Inode>() <= block_data.len() {
                    let inode = Inode::from_bytes(&block_data[offset..])?;
                    inodes.push(inode);
                }
            }
        }
        
        Ok(InodeTable { inodes })
    }

    pub fn to_disk(&self, drive: usize, super_block: &SuperBlock, bitmap: &mut BlockBitmap) -> Result<(), InodeError> {
        let inode_size = core::mem::size_of::<Inode>() as u64;
        let inodes_per_block = super_block.block_size / inode_size;
        
        for (block_idx, block) in (0..super_block.inode_table_blocks).enumerate() {
            let mut block_data = Vec::new();
            
            for j in 0..inodes_per_block {
                let inode_idx = block_idx as u64 * inodes_per_block + j;
                if (inode_idx as usize) < self.inodes.len() {
                    block_data.extend_from_slice(&self.inodes[inode_idx as usize].to_bytes());
                } else {
                    block_data.extend_from_slice(&[0u8; core::mem::size_of::<Inode>()]);
                }
            }
            
            write_block(drive, super_block, bitmap, super_block.inode_table_start + block as u64, &block_data)?;
        }
        
        Ok(())
    }
}

pub struct InodeManager {
    pub drive: usize,
    pub super_block: SuperBlock,
    pub bitmap: BlockBitmap,
    pub inode_table: InodeTable,
}

impl InodeManager {
    pub fn new(drive: usize, super_block: SuperBlock, bitmap: BlockBitmap) -> Result<Self, InodeError> {
        let inode_table = InodeTable::from_disk(drive, &super_block)?;
        
        Ok(InodeManager {
            drive,
            super_block,
            bitmap,
            inode_table,
        })
    }

    pub fn create_inode(&mut self) -> Result<u16, InodeError> {
        let inode = Inode::new();
        self.inode_table.inodes.push(inode);
        Ok((self.inode_table.inodes.len() - 1) as u16)
    }

    pub fn read_inode(&self, inode_index: u16) -> Result<Inode, InodeError> {
        if inode_index as usize >= self.inode_table.inodes.len() {
            return Err(InodeError::OutOfBounds);
        }
        Ok(self.inode_table.inodes[inode_index as usize])
    }

    pub fn write_inode(&mut self, inode_index: u16, inode: Inode) -> Result<(), InodeError> {
        if inode_index as usize >= self.inode_table.inodes.len() {
            return Err(InodeError::OutOfBounds);
        }
        self.inode_table.inodes[inode_index as usize] = inode;
        self.inode_table.to_disk(self.drive, &self.super_block, &mut self.bitmap)
    }

    pub fn allocate_block_to_inode(&mut self, inode_index: u16) -> Result<u64, InodeError> {
        let mut inode = self.read_inode(inode_index)?;
        
        for direct_block in inode.direct_blocks.iter_mut() {
            if *direct_block == 0 {
                match self.bitmap.allocate_block() {
                    Ok(block) => {
                        *direct_block = block;
                        self.write_inode(inode_index, inode)?;
                        println!("Allocated block {} to inode {}", block, inode_index);
                        return Ok(block);
                    }
                    Err(e) => return Err(InodeError::BitmapError(e)),
                }
            }
        }
        
        Err(InodeError::OutOfBounds)
    }

    pub fn save(&mut self) -> Result<(), InodeError> {
        self.inode_table.to_disk(self.drive, &self.super_block, &mut self.bitmap)
    }
}

pub struct FileHandle {
    pub inode_index: u16,
    pub position: u64,
}

pub struct DirectoryHandle {
    pub inode_index: u16,
    pub position: u64,
}

impl fmt::Display for Inode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Inode(size: {}, direct_blocks: {:?}, indirect_block: {}, double_indirect_block: {})",
            self.size,
            self.direct_blocks,
            self.indirect_block,
            self.double_indirect_block)
    }
}

impl fmt::Display for InodeTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InodeTable(num_inodes: {})", self.inodes.len())
    }
}