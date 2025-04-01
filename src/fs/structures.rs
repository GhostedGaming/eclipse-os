use alloc::string::String;
use alloc::vec::Vec;
use core::mem;

// Magic number to identify EclipseFS
pub const ECLIPSE_FS_MAGIC: u32 = 0x45434C50; // "ECLP" in ASCII

// Block size (4KB)
pub const BLOCK_SIZE: usize = 4096;

// Superblock structure - stores filesystem metadata
#[repr(C, packed)]
pub struct Superblock {
    pub magic: u32,              // Magic number to identify the filesystem
    pub version: u16,            // Filesystem version
    pub block_size: u32,         // Size of blocks in bytes
    pub total_blocks: u64,       // Total number of blocks
    pub free_blocks: u64,        // Number of free blocks
    pub total_inodes: u32,       // Total number of inodes
    pub free_inodes: u32,        // Number of free inodes
    pub created_time: u64,       // Creation timestamp
    pub last_mount_time: u64,    // Last mount timestamp
    pub last_write_time: u64,    // Last write timestamp
    pub mount_count: u16,        // Number of mounts
    pub max_mount_count: u16,    // Maximum mount count before check
    pub state: u8,               // Filesystem state (clean/dirty)
    pub reserved: [u8; 64],      // Reserved for future use
}

// Inode structure - stores file metadata
#[repr(C, packed)]
pub struct Inode {
    pub mode: u16,               // File type and permissions
    pub uid: u16,                // Owner user ID
    pub gid: u16,                // Owner group ID
    pub flags: u32,              // Flags
    pub size: u64,               // Size in bytes
    pub access_time: u64,        // Last access time
    pub creation_time: u64,      // Creation time
    pub modification_time: u64,  // Last modification time
    pub link_count: u16,         // Number of hard links
    pub direct_blocks: [u32; 12], // Direct block pointers
    pub indirect_block: u32,     // Single indirect block pointer
    pub double_indirect: u32,    // Double indirect block pointer
    pub triple_indirect: u32,    // Triple indirect block pointer
    pub reserved: [u8; 24],      // Reserved for future use
}

// Directory entry structure
#[repr(C, packed)]
pub struct DirEntry {
    pub inode: u32,              // Inode number
    pub rec_len: u16,            // Record length
    pub name_len: u8,            // Name length
    pub file_type: u8,           // File type
}

// File types
pub const FILE_TYPE_UNKNOWN: u8 = 0;
pub const FILE_TYPE_REGULAR: u8 = 1;
pub const FILE_TYPE_DIRECTORY: u8 = 2;
pub const FILE_TYPE_SYMLINK: u8 = 3;

// Filesystem states
pub const FS_STATE_CLEAN: u8 = 0;
pub const FS_STATE_DIRTY: u8 = 1;