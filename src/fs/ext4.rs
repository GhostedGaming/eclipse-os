use core::slice;

use super::Ext4Superblock;

pub fn write_super_block() {
    let mut super_block = Ext4Superblock {
        s_magic_number: 0xEF53,
        s_state: 1,
        ..Default::default()
    };

    let superblock_bytes = unsafe {
        slice::from_raw_parts(&super_block as *const Ext4Superblock as *const u8, core::mem::size_of::<Ext4Superblock>())
    }; 
}