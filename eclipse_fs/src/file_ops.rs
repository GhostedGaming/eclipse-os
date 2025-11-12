use crate::block_io::{read_block, write_block};
use crate::inodes::{InodeManager, InodeError};

use alloc::{vec, vec::Vec};
use eclipse_framebuffer::println;

pub fn create_file(
    inode_manager: &mut InodeManager,
    data: &[u8],
) -> Result<u16, InodeError> {
    let inode_index = inode_manager.create_inode()?;
    let block_size = inode_manager.super_block.block_size as usize;
    
    println!("Creating file: inode {}, size {} bytes", inode_index, data.len());
    
    let mut inode = inode_manager.read_inode(inode_index)?;
    inode.size = data.len() as u64;
    
    for (i, chunk) in data.chunks(block_size).enumerate() {
        if i < 12 {
            let block = inode_manager.allocate_block_to_inode(inode_index)?;
            inode.direct_blocks[i] = block;
            write_block(
                inode_manager.drive,
                &inode_manager.super_block,
                &mut inode_manager.bitmap,
                block,
                chunk,
            )?;
            println!("File block {} written to direct block {}", i, block);
        } else {
            handle_indirect_block(inode_manager, &mut inode, i, chunk)?;
            println!("File block {} written to indirect block", i);
        }
    }
    
    inode_manager.write_inode(inode_index, inode)?;
    println!("File created successfully at inode {}", inode_index);
    
    Ok(inode_index)
}

pub fn read_file(
    inode_manager: &InodeManager,
    inode_index: u16,
) -> Result<Vec<u8>, InodeError> {
    let inode = inode_manager.read_inode(inode_index)?;
    let block_size = inode_manager.super_block.block_size as usize;
    let mut file_data = Vec::with_capacity(inode.size as usize);
    
    println!("Reading file: inode {}, size {} bytes", inode_index, inode.size);
    
    let blocks_to_read = (inode.size as usize + block_size - 1) / block_size;
    
    for (i, &block_num) in inode.direct_blocks.iter().enumerate() {
        if block_num == 0 || i >= blocks_to_read {
            break;
        }
        
        let block_data = read_block(
            inode_manager.drive,
            &inode_manager.super_block,
            &inode_manager.bitmap,
            block_num,
        )?;
        
        let remaining = inode.size as usize - file_data.len();
        let to_copy = core::cmp::min(remaining, block_data.len());
        
        file_data.extend_from_slice(&block_data[..to_copy]);
        println!("Read file block {}: {} bytes from direct block {}", i, to_copy, block_num);
        
        if file_data.len() >= inode.size as usize {
            return Ok(file_data);
        }
    }
    
    if inode.indirect_block != 0 && blocks_to_read > 12 {
        read_indirect_blocks(inode_manager, &inode, &mut file_data, blocks_to_read)?;
    }
    
    println!("File read successfully: {} bytes", file_data.len());
    Ok(file_data)
}

pub fn list_files(
    inode_manager: &InodeManager,
) -> Result<Vec<u16>, InodeError> {
    let mut file_inodes = Vec::new();
    
    println!("Listing all files in the filesystem");
    
    for (i, inode) in inode_manager.inode_table.inodes.iter().enumerate() {
        if inode.size > 0 {
            file_inodes.push(i as u16);
            println!("Found file: inode {}, size {} bytes", i, inode.size);
        }
    }
    
    Ok(file_inodes)
}

pub fn delete_file(
    inode_manager: &mut InodeManager,
    inode_index: u16,
) -> Result<(), InodeError> {
    let inode = inode_manager.read_inode(inode_index)?;
    
    println!("Deleting file: inode {}, size {} bytes", inode_index, inode.size);
    
    for &block_num in inode.direct_blocks.iter() {
        if block_num != 0 {
            inode_manager.bitmap.free_block(block_num)?;
            println!("Freed direct block {}", block_num);
        }
    }
    
    if inode.indirect_block != 0 {
        free_indirect_blocks(inode_manager, &inode)?;
    }
    
    let mut empty_inode = inode_manager.read_inode(inode_index)?;
    empty_inode.size = 0;
    empty_inode.direct_blocks = [0; 12];
    empty_inode.indirect_block = 0;
    empty_inode.double_indirect_block = 0;
    
    inode_manager.write_inode(inode_index, empty_inode)?;
    println!("File deleted successfully");
    
    Ok(())
}

fn handle_indirect_block(
    inode_manager: &mut InodeManager,
    inode: &mut crate::inodes::Inode,
    block_index: usize,
    data: &[u8],
) -> Result<(), InodeError> {
    let block_size = inode_manager.super_block.block_size as usize;
    let blocks_per_indirect = block_size / 8;
    
    if inode.indirect_block == 0 {
        inode.indirect_block = inode_manager.bitmap.allocate_block()?;
        println!("Allocated indirect block at {}", inode.indirect_block);
    }
    
    let indirect_index = (block_index - 12) % blocks_per_indirect;
    let indirect_offset = indirect_index * 8;
    
    let data_block = inode_manager.bitmap.allocate_block()?;
    write_block(
        inode_manager.drive,
        &inode_manager.super_block,
        &mut inode_manager.bitmap,
        data_block,
        data,
    )?;
    
    let mut indirect_data = vec![0u8; block_size];
    let ptr = &data_block as *const u64 as *const u8;
    unsafe {
        core::ptr::copy_nonoverlapping(
            ptr,
            indirect_data.as_mut_ptr().add(indirect_offset),
            8,
        );
    }
    
    write_block(
        inode_manager.drive,
        &inode_manager.super_block,
        &mut inode_manager.bitmap,
        inode.indirect_block,
        &indirect_data,
    )?;
    
    Ok(())
}

fn read_indirect_blocks(
    inode_manager: &InodeManager,
    inode: &crate::inodes::Inode,
    file_data: &mut Vec<u8>,
    blocks_to_read: usize,
) -> Result<(), InodeError> {
    let block_size = inode_manager.super_block.block_size as usize;
    let blocks_per_indirect = block_size / 8;
    
    let indirect_data = read_block(
        inode_manager.drive,
        &inode_manager.super_block,
        &inode_manager.bitmap,
        inode.indirect_block,
    )?;
    
    for i in 12..blocks_to_read {
        let indirect_idx = (i - 12) % blocks_per_indirect;
        let offset = indirect_idx * 8;
        
        if offset + 8 > indirect_data.len() {
            break;
        }
        
        let block_num = u64::from_le_bytes([
            indirect_data[offset], indirect_data[offset + 1],
            indirect_data[offset + 2], indirect_data[offset + 3],
            indirect_data[offset + 4], indirect_data[offset + 5],
            indirect_data[offset + 6], indirect_data[offset + 7],
        ]);
        
        if block_num == 0 {
            break;
        }
        
        let block_data = read_block(
            inode_manager.drive,
            &inode_manager.super_block,
            &inode_manager.bitmap,
            block_num,
        )?;
        
        let remaining = inode.size as usize - file_data.len();
        let to_copy = core::cmp::min(remaining, block_data.len());
        
        file_data.extend_from_slice(&block_data[..to_copy]);
        println!("Read file block {}: {} bytes from indirect block {}", i, to_copy, block_num);
    }
    
    Ok(())
}

fn free_indirect_blocks(
    inode_manager: &mut InodeManager,
    inode: &crate::inodes::Inode,
) -> Result<(), InodeError> {
    let indirect_data = read_block(
        inode_manager.drive,
        &inode_manager.super_block,
        &inode_manager.bitmap,
        inode.indirect_block,
    )?;
    
    for offset in (0..indirect_data.len()).step_by(8) {
        if offset + 8 > indirect_data.len() {
            break;
        }
        
        let block_num = u64::from_le_bytes([
            indirect_data[offset], indirect_data[offset + 1],
            indirect_data[offset + 2], indirect_data[offset + 3],
            indirect_data[offset + 4], indirect_data[offset + 5],
            indirect_data[offset + 6], indirect_data[offset + 7],
        ]);
        
        if block_num != 0 {
            inode_manager.bitmap.free_block(block_num)?;
            println!("Freed indirect data block {}", block_num);
        }
    }
    
    inode_manager.bitmap.free_block(inode.indirect_block)?;
    println!("Freed indirect block {}", inode.indirect_block);
    
    Ok(())
}