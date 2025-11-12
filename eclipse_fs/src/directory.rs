use crate::inodes::{InodeManager, InodeError};
use alloc::vec::Vec;
use eclipse_framebuffer::println;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DirectoryEntry {
    pub inode_number: u16,
    pub name: [u8; 256],
    pub name_len: u8,
}

impl DirectoryEntry {
    pub fn new(inode_number: u16, name: &[u8]) -> Self {
        let mut entry = DirectoryEntry {
            inode_number,
            name: [0u8; 256],
            name_len: core::cmp::min(name.len(), 255) as u8,
        };
        entry.name[..entry.name_len as usize].copy_from_slice(&name[..entry.name_len as usize]);
        entry
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let ptr = self as *const DirectoryEntry as *const u8;
        unsafe {
            Vec::from(core::slice::from_raw_parts(ptr, core::mem::size_of::<DirectoryEntry>()))
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, InodeError> {
        if bytes.len() < core::mem::size_of::<DirectoryEntry>() {
            return Err(InodeError::ReadFailed);
        }
        unsafe {
            let mut entry = DirectoryEntry {
                inode_number: 0,
                name: [0u8; 256],
                name_len: 0,
            };
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), &mut entry as *mut _ as *mut u8, core::mem::size_of::<DirectoryEntry>());
            Ok(entry)
        }
    }
}

pub struct DirectoryManager;

impl DirectoryManager {
    pub fn create_directory(
        inode_manager: &mut InodeManager,
    ) -> Result<u16, InodeError> {
        let inode_index = inode_manager.create_inode()?;
        println!("Created directory at inode {}", inode_index);
        Ok(inode_index)
    }

    pub fn add_entry(
        inode_manager: &mut InodeManager,
        dir_inode_index: u16,
        name: &[u8],
        target_inode: u16,
    ) -> Result<(), InodeError> {
        let entry = DirectoryEntry::new(target_inode, name);
        let entry_bytes = entry.to_bytes();
        
        let mut dir_inode = inode_manager.read_inode(dir_inode_index)?;
        dir_inode.size += entry_bytes.len() as u64;
        
        println!("Adding entry '{}' -> inode {} to directory {}", 
            core::str::from_utf8(name).unwrap_or("invalid_utf8"),
            target_inode, 
            dir_inode_index
        );
        
        inode_manager.write_inode(dir_inode_index, dir_inode)?;
        Ok(())
    }

    pub fn find_entry(
        inode_manager: &InodeManager,
        dir_inode_index: u16,
        name: &[u8],
    ) -> Result<Option<u16>, InodeError> {
        let dir_inode = inode_manager.read_inode(dir_inode_index)?;
        let block_size = inode_manager.super_block.block_size as usize;
        let entries_per_block = block_size / core::mem::size_of::<DirectoryEntry>();
        
        println!("Searching for '{}' in directory {}", 
            core::str::from_utf8(name).unwrap_or("invalid_utf8"),
            dir_inode_index
        );
        
        for block_idx in 0..12 {
            if dir_inode.direct_blocks[block_idx] == 0 {
                break;
            }
            
            let block_data = crate::block_io::read_block(
                inode_manager.drive,
                &inode_manager.super_block,
                &inode_manager.bitmap,
                dir_inode.direct_blocks[block_idx],
            )?;
            
            for entry_idx in 0..entries_per_block {
                let offset = entry_idx * core::mem::size_of::<DirectoryEntry>();
                if offset + core::mem::size_of::<DirectoryEntry>() > block_data.len() {
                    break;
                }
                
                let entry = DirectoryEntry::from_bytes(&block_data[offset..])?;
                
                if entry.inode_number != 0 {
                    let entry_name = &entry.name[..entry.name_len as usize];
                    if entry_name == name {
                        println!("Found entry '{}' at inode {}", 
                            core::str::from_utf8(name).unwrap_or("invalid_utf8"),
                            entry.inode_number
                        );
                        return Ok(Some(entry.inode_number));
                    }
                }
            }
        }
        
        println!("Entry {} not found in directory {}", 
            core::str::from_utf8(name).unwrap_or("invalid_utf8"),
            dir_inode_index
        );
        Ok(None)
    }

    pub fn list_directory(
        inode_manager: &InodeManager,
        dir_inode_index: u16,
    ) -> Result<Vec<(u16, Vec<u8>)>, InodeError> {
        let dir_inode = inode_manager.read_inode(dir_inode_index)?;
        let block_size = inode_manager.super_block.block_size as usize;
        let entries_per_block = block_size / core::mem::size_of::<DirectoryEntry>();
        let mut entries = Vec::new();
        
        println!("Listing directory {}", dir_inode_index);
        
        for block_idx in 0..12 {
            if dir_inode.direct_blocks[block_idx] == 0 {
                break;
            }
            
            let block_data = crate::block_io::read_block(
                inode_manager.drive,
                &inode_manager.super_block,
                &inode_manager.bitmap,
                dir_inode.direct_blocks[block_idx],
            )?;
            
            for entry_idx in 0..entries_per_block {
                let offset = entry_idx * core::mem::size_of::<DirectoryEntry>();
                if offset + core::mem::size_of::<DirectoryEntry>() > block_data.len() {
                    break;
                }
                
                let entry = DirectoryEntry::from_bytes(&block_data[offset..])?;
                
                if entry.inode_number != 0 {
                    let name_slice = &entry.name[..entry.name_len as usize];
                    let name_vec = name_slice.to_vec();
                    entries.push((entry.inode_number, name_vec.clone()));
                    println!("  {} (inode {})", 
                        core::str::from_utf8(name_slice).unwrap_or("invalid_utf8"),
                        entry.inode_number
                    );
                }
            }
        }
        
        println!("Directory contains {} entries", entries.len());
        Ok(entries)
    }
}
