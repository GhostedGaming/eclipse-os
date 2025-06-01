pub mod ext4;

/// Information for the disk
#[repr(C)]
#[derive(Debug, Default)]
struct Ext4Superblock {
    s_inodes_count: u32,      // Total number of inodes
    s_blocks_count: u32,      // Total number of blocks
    s_r_blocks_count: u32,    // Reserved blocks count
    s_free_blocks_count: u32, // Free blocks count
    s_free_inodes_count: u32, // Free inodes count
    s_first_data_block: u32,  // First data block
    s_log_block_size: u32,    // Block size
    s_blocks_per_group: u32,  // Blocks per group
    s_inodes_per_group: u32,  // Inodes per group
    s_mtime: u32,             // Last mount time
    s_wtime: u32,             // Last write time
    s_magic_number: u16,      // Ext4 magic signature (0xEF53)
    s_state: u16,             // Filesystem state
    s_feature_compat: u32,    // Compatible feature flags
    s_feature_incompat: u32,  // Incompatible feature flags
    s_feature_ro_compat: u32, // Read-only compatible features
    s_uuid: [u8; 16],         // Filesystem UUID
    s_volume_name: [u8; 16],  // Volume name
    s_journal_uuid: [u8; 16], // Journal UUID
    s_inode_size: u16,        // Size of each inode structure
    s_checksum: u32,          // Superblock checksum
}
