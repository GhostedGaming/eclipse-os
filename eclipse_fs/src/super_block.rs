use core::fmt;
use eclipse_ide::IDE_DEVICES;
use eclipse_framebuffer::println;

/// Superblock structure
pub struct SuperBlock {
    magic: u16,
    version: u8,
    size: u64,
    pub block_size: u64,
    blocks: u64,
    inodes: u16,
    reserved: u16,
    
    pub superblock_blocks: u64,
    pub inode_table_start: u64,
    pub inode_table_blocks: u64,
    pub block_bitmap_start: u64,
    pub block_bitmap_blocks: u64,
    pub data_region_start: u64,
    pub reserved_start: u64,
    pub reserved_blocks: u64,
}

impl SuperBlock {
    const MAGIC: u16 = 0xEC1;
    const VERSION: u8 = 1;
    const DEFAULT_INODES: u16 = 500;
    const RESERVED: u16 = 500;
    const INODE_SIZE: u64 = 128;
    
    pub fn new(drive: u8) -> Self {
        let sector_count = unsafe { IDE_DEVICES[drive as usize].size };
        let size_bytes: u64 = sector_count * 512;
        
        if size_bytes == 0 {
            println!("Warning: Drive {} has size 0", drive);
        }
        
        let block_size: u64 = match size_bytes {
            0..=16_000_000_000_000 => 4 * 1024,          // 4 KiB (default)
            16_000_000_000_001..=32_000_000_000_000 => 8 * 1024,   // 8 KiB
            32_000_000_000_001..=64_000_000_000_000 => 16 * 1024,  // 16 KiB
            64_000_000_000_001..=128_000_000_000_000 => 32 * 1024, // 32 KiB
            128_000_000_000_001..=256_000_000_000_000 => 64 * 1024, // 64 KiB
            256_000_000_000_001..=512_000_000_000_000 => 128 * 1024, // 128 KiB
            512_000_000_000_001..=1_000_000_000_000_000 => 256 * 1024, // 256 KiB
            1_000_000_000_000_001..=2_000_000_000_000_000 => 512 * 1024, // 512 KiB
            2_000_000_000_000_001..=4_000_000_000_000_000 => 1024 * 1024, // 1024 KiB
            _ => 2048 * 1024, // 2048 KiB (max size)
        };
        
        let blocks = size_bytes / block_size;
        
        let superblock_blocks = 1;
        
        let inode_table_start = superblock_blocks;
        let inode_table_size_bytes = Self::DEFAULT_INODES as u64 * Self::INODE_SIZE;
        let inode_table_blocks = (inode_table_size_bytes + block_size - 1) / block_size;

        let block_bitmap_start = inode_table_start + inode_table_blocks;
        let bitmap_bytes_needed = (blocks + 7) / 8;
        let block_bitmap_blocks =  (bitmap_bytes_needed + block_size - 1) / block_size;
        
        let reserved_start = block_bitmap_start + block_bitmap_blocks; 
        let reserved_blocks = Self::RESERVED as u64;
        
        let data_region_start = reserved_start + reserved_blocks;
        
        Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
            size: size_bytes,
            block_size,
            blocks,
            inodes: Self::DEFAULT_INODES,
            reserved: Self::RESERVED,
            superblock_blocks,
            inode_table_start,
            inode_table_blocks,
            block_bitmap_start,
            block_bitmap_blocks,
            data_region_start,
            reserved_start,
            reserved_blocks,
        }
    }
    
    pub fn to_bytes(&self) -> [u8; 512] {
        let mut bytes = [0u8; 512];
        
        bytes[0..2].copy_from_slice(&self.magic.to_le_bytes());
        bytes[2] = self.version;
        bytes[8..16].copy_from_slice(&self.size.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.block_size.to_le_bytes());
        bytes[24..32].copy_from_slice(&self.blocks.to_le_bytes());
        bytes[32..34].copy_from_slice(&self.inodes.to_le_bytes());
        bytes[34..36].copy_from_slice(&self.reserved.to_le_bytes());
        bytes[40..48].copy_from_slice(&self.superblock_blocks.to_le_bytes());
        bytes[48..56].copy_from_slice(&self.inode_table_start.to_le_bytes());
        bytes[56..64].copy_from_slice(&self.inode_table_blocks.to_le_bytes());
        
        bytes[64..72].copy_from_slice(&self.block_bitmap_start.to_le_bytes());
        bytes[72..80].copy_from_slice(&self.block_bitmap_blocks.to_le_bytes());
        
        bytes[80..88].copy_from_slice(&self.data_region_start.to_le_bytes());
        bytes[88..96].copy_from_slice(&self.reserved_start.to_le_bytes());
        bytes[96..104].copy_from_slice(&self.reserved_blocks.to_le_bytes());
        
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 104 { 
            return Err("Buffer too small");
        }
        
        let magic = u16::from_le_bytes([bytes[0], bytes[1]]);
        if magic != Self::MAGIC {
            return Err("Invalid magic number");
        }
        
        let version = bytes[2];
        let size = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let block_size = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        let blocks = u64::from_le_bytes(bytes[24..32].try_into().unwrap());
        let inodes = u16::from_le_bytes([bytes[32], bytes[33]]);
        let reserved = u16::from_le_bytes([bytes[34], bytes[35]]);
        let superblock_blocks = u64::from_le_bytes(bytes[40..48].try_into().unwrap());
        let inode_table_start = u64::from_le_bytes(bytes[48..56].try_into().unwrap());
        let inode_table_blocks = u64::from_le_bytes(bytes[56..64].try_into().unwrap());

        let block_bitmap_start = u64::from_le_bytes(bytes[64..72].try_into().unwrap());
        let block_bitmap_blocks = u64::from_le_bytes(bytes[72..80].try_into().unwrap());
        
        let data_region_start = u64::from_le_bytes(bytes[80..88].try_into().unwrap());
        let reserved_start = u64::from_le_bytes(bytes[88..96].try_into().unwrap());
        let reserved_blocks = u64::from_le_bytes(bytes[96..104].try_into().unwrap());
        
        Ok(Self {
            magic,
            version,
            size,
            block_size,
            blocks,
            inodes,
            reserved,
            superblock_blocks,
            inode_table_start,
            inode_table_blocks,
            block_bitmap_start,
            block_bitmap_blocks,
            data_region_start,
            reserved_start,
            reserved_blocks,
        })
    }
}

impl fmt::Display for SuperBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SuperBlock {{ magic: 0x{:X}, version: {}, size: {} bytes, block_size: {}, blocks: {}, inodes: {}, inode_table: blocks {}-{}, block_bitmap: blocks {}-{}, reserved: blocks {}-{}, data_start: block {} }}",
            self.magic, self.version, self.size, self.block_size, self.blocks, self.inodes,
            self.inode_table_start, self.inode_table_start + self.inode_table_blocks - 1,
            self.block_bitmap_start, self.block_bitmap_start + self.block_bitmap_blocks - 1, // Added bitmap info
            self.reserved_start, self.reserved_start + self.reserved_blocks - 1,
            self.data_region_start
        )
    }
}
