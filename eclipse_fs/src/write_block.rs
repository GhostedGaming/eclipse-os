use eclipse_ide::{ide_write_sectors, ide_read_sectors};

fn write_block(drive: u8, lba: u32, buffer: *const u8) {
    ide_write_sectors(drive, lba, buffer);
}