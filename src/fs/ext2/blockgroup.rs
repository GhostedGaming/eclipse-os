pub struct BlockGrup {
    pub block_usage_bitmap: u32,
    pub inode_usage_bitmap: u32,
    pub inode_table_addr: u32,
    pub num_unallocated_blocks: u16,
    pub num_unallocated_inodes: u16,
    pub num_dirs_in_group: u16,
    pub unused: [u8; 14],
}