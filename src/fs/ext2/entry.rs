use crate::alloc::vec::Vec;

pub struct Entry {
    inode: u32,
    total_size: u16,
    lower_size: u8,
    upper_size: u8,
    name: Vec<u8>,
}
