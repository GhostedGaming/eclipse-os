extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

// BootSector struct and implementation
#[repr(C, packed)]
#[derive(Clone, Copy)] // Add Clone and Copy traits
pub struct BootSector {
    pub jump_boot: [u8; 3],
    pub oem_name: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sector_count: u16,
    pub num_fats: u8,
    pub root_entry_count: u16,
    pub total_sectors_16: u16,
    pub media: u8,
    pub fat_size_16: u16,
    pub sectors_per_track: u16,
    pub num_heads: u16,
    pub hidden_sectors: u32,
    pub total_sectors_32: u32,
    pub fat_size_32: u32,
    pub ext_flags: u16,
    pub fs_version: u16,
    pub root_cluster: u32,
    pub fs_info: u16,
    pub backup_boot_sector: u16,
    pub reserved: [u8; 12],
    pub drive_number: u8,
    pub reserved1: u8,
    pub boot_signature: u8,
    pub volume_id: u32,
    pub volume_label: [u8; 11],
    pub fs_type: [u8; 8],
}

impl BootSector {
    pub fn parse(data: &[u8]) -> Self {
        unsafe { *(data.as_ptr() as *const BootSector) }
    }
}

// DirectoryEntry struct and implementation
#[repr(C, packed)]
#[derive(Clone, Copy)] // Add Clone and Copy traits
pub struct DirectoryEntry {
    pub name: [u8; 11],
    pub attr: u8,
    pub reserved: u8,
    pub create_time_tenth: u8,
    pub create_time: u16,
    pub create_date: u16,
    pub last_access_date: u16,
    pub first_cluster_high: u16,
    pub write_time: u16,
    pub write_date: u16,
    pub first_cluster_low: u16,
    pub file_size: u32,
}

impl DirectoryEntry {
    pub fn parse(data: &[u8]) -> Self {
        unsafe { *(data.as_ptr() as *const DirectoryEntry) }
    }

    pub fn is_file(&self) -> bool {
        self.attr & 0x10 == 0
    }

    pub fn is_directory(&self) -> bool {
        self.attr & 0x10 != 0
    }

    pub fn get_name(&self) -> String {
        let name = core::str::from_utf8(&self.name).unwrap_or("").trim();
        name.to_string()
    }
}

// FileSystem struct and implementation
pub struct FileSystem {
    pub cluster_size: usize,
    pub root_cluster: u32,
    pub data_area_offset: usize,
}

impl FileSystem {
    pub fn list_directory(&self, cluster: u32, read_sector: fn(u32, &mut [u8])) -> Vec<String> {
        let mut entries = Vec::new();
        let mut buffer = [0u8; 512];
        let mut current_cluster = cluster;

        loop {
            let sector = self.cluster_to_sector(current_cluster);
            read_sector(sector, &mut buffer);

            for i in 0..(self.cluster_size / core::mem::size_of::<DirectoryEntry>()) {
                let entry_offset = i * core::mem::size_of::<DirectoryEntry>();
                let entry_data = &buffer[entry_offset..entry_offset + core::mem::size_of::<DirectoryEntry>()];
                let entry = DirectoryEntry::parse(entry_data);

                if entry.name[0] == 0x00 || entry.name[0] == 0xE5 {
                    continue;
                }

                entries.push(entry.get_name());
            }

            current_cluster = self.next_cluster(current_cluster);
            if current_cluster == 0x0FFFFFFF {
                break;
            }
        }

        entries
    }

    fn cluster_to_sector(&self, cluster: u32) -> u32 {
        self.data_area_offset as u32 + (cluster - 2) * self.cluster_size as u32
    }

    fn next_cluster(&self, _cluster: u32) -> u32 {
        0x0FFFFFFF
    }
}