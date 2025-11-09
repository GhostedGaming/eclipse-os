use core::fmt;
use eclipse_ide::{IDE_DEVICES, IdeDevice};
use eclipse_framebuffer::println;

struct SuperBlock {
    magic: u16,
    version: u8,
    size: u64,
    block_size: u64,
    blocks: u64,
    inodes: u16,
}

impl SuperBlock {
    pub fn new(drive: u8) -> Option<Self> {
        unsafe {
            let size_bytes: u64 = (IDE_DEVICES[drive as usize].size * 512).into();
            
            if size_bytes == 0 {
                println!("Drive size 0");
                return None;
            }
            
            Some(Self {
                magic: 0xEC1,            // Magic ECL
                version: 1,              // FS version
                size: size_bytes,
                block_size: 16 * 1024,   // 16 KiB
                blocks: size_bytes / (16 * 1024),
                inodes: 500,
            })
        }
    }
}

impl fmt::Display for SuperBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SuperBlock {{ magic: 0x{:X}, version: {}, size: {} bytes, block_size: {}, blocks: {}, inodes: {} }}",
            self.magic, self.version, self.size, self.block_size, self.blocks, self.inodes
        )
    }
}