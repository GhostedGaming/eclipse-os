use alloc::vec::{self, Vec};
use alloc::string::{String, ToString};
use crate::fs::list_disks;
use crate::println;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiskType {
    Hda,
    Sata,
    Nvme,
    Virtio,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Disk {
    pub id: usize,
    pub disk_type: DiskType,
    pub name: String,
    pub size: u64,
}

pub fn list_disks() {
    let disks = detect_disks();

    println!("Disk Information:");
    println!("----------------");

    if disks.is_empty() {
        println!("No disks detected");
    } 

    for (i, disk) in disks.iter().enumerate() {
        println!("{}. {} ({:?}, {}MB,)", 
            i+1, 
            disk.name, 
            disk.disk_type, 
            disk.size / (1024 * 1024), 
        );
    }
}

fn detect_disks() -> Vec<Disk> {
    let mut disks = Vec::new();

    disks.push(Disk {
        id: 0,
        disk_type: DiskType::Hda,
        name: "hda".to_string(),
        size: 128 * 1024 * 1024,
    });

    disks.push(Disk {
        id: 1,
        disk_type: DiskType::Sata,
        name: "sda".to_string(),
        size: 2 * 1024 * 1024 * 1024,
    });

    disks.push(Disk {
        id: 2,
        disk_type: DiskType::Virtio,
        name: "vda".to_string(),
        size: 4 * 1024 * 1024 * 1024, // 4GB
    });

    disks.push(Disk {
        id: 3,
        disk_type: DiskType::Nvme,
        name: "nvme".to_string(),
        size: 6 * 1024 * 1024 * 1024,
    });

    disks.push(Disk {
        id: 4,
        disk_type: DiskType::Unknown,
        name: "unknown".to_string(),
        size: 8 * 1024 * 1024 * 1024,
    });

    disks
}
