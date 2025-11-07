use eclipse_ide::IDE_DEVICES;

struct SuperBlock {
    magic: u16,
    version: u8,
    size: u64,
    block_size: u64,
    blocks: u64,
    inodes: u8,
}

impl SuperBlock {
    pub fn new(drive: u8) -> Self {
        unsafe {
            let size_bytes = IDE_DEVICES[drive as usize].size * 512; // total bytes

            Self {
                magic: 0xEC1, // Magic ECL
                version: 1,   // FS version
                size: size_bytes as u64,
                block_size: 16 * 1024, // 16 KiB
                blocks: size_bytes as u64 / (16 * 1024),
                inodes: 0,
            }
        }
    }
}
