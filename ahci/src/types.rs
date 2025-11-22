#![no_std]

use core::ptr::{read_volatile, write_volatile};

#[repr(C)]
pub struct HbaPort {
    pub clb: u64,
    pub fb: u64,
    pub is: u32,
    pub ie: u32,
    pub cmd: u32,
    reserved0: u32,
    pub tfd: u32,
    pub sig: u32,
    pub ssts: u32,
    pub sctl: u32,
    pub serr: u32,
    pub sact: u32,
    pub ci: u32,
    pub sntf: u32,
    pub fbs: u32,
    reserved1: [u32; 11],
    vendor: [u32; 4],
}

impl HbaPort {
    pub fn read_cmd(&self) -> u32 {
        unsafe { read_volatile(&self.cmd) }
    }

    pub fn write_cmd(&mut self, value: u32) {
        unsafe { write_volatile(&mut self.cmd, value) }
    }

    pub fn read_is(&self) -> u32 {
        unsafe { read_volatile(&self.is) }
    }

    pub fn write_is(&mut self, value: u32) {
        unsafe { write_volatile(&mut self.is, value) }
    }

    pub fn read_tfd(&self) -> u32 {
        unsafe { read_volatile(&self.tfd) }
    }

    pub fn read_ssts(&self) -> u32 {
        unsafe { read_volatile(&self.ssts) }
    }

    pub fn read_sig(&self) -> u32 {
        unsafe { read_volatile(&self.sig) }
    }

    pub fn read_ci(&self) -> u32 {
        unsafe { read_volatile(&self.ci) }
    }

    pub fn write_ci(&mut self, value: u32) {
        unsafe { write_volatile(&mut self.ci, value) }
    }

    pub fn write_serr(&mut self, value: u32) {
        unsafe { write_volatile(&mut self.serr, value) }
    }
}

#[repr(C)]
pub struct HbaMem {
    pub cap: u32,
    pub ghc: u32,
    pub is: u32,
    pub pi: u32,
    pub vs: u32,
    pub ccc_ctl: u32,
    pub ccc_pts: u32,
    pub em_loc: u32,
    pub em_ctl: u32,
    pub cap2: u32,
    pub bohc: u32,
    reserved: [u8; 0xA0 - 0x2C],
    vendor: [u8; 0x100 - 0xA0],
    pub ports: [HbaPort; 32],
}

impl HbaMem {
    pub fn read_cap(&self) -> u32 {
        unsafe { read_volatile(&self.cap) }
    }

    pub fn read_ghc(&self) -> u32 {
        unsafe { read_volatile(&self.ghc) }
    }

    pub fn write_ghc(&mut self, value: u32) {
        unsafe { write_volatile(&mut self.ghc, value) }
    }

    pub fn read_pi(&self) -> u32 {
        unsafe { read_volatile(&self.pi) }
    }

    pub fn read_is(&self) -> u32 {
        unsafe { read_volatile(&self.is) }
    }

    pub fn write_is(&mut self, value: u32) {
        unsafe { write_volatile(&mut self.is, value) }
    }
}

#[repr(C)]
pub struct HbaCmdHeader {
    pub flags: u16,
    pub prdtl: u16,
    pub prdbc: u32,
    pub ctba: u64,
    reserved: [u32; 4],
}

#[repr(C)]
pub struct HbaPrdtEntry {
    pub dba: u64,
    reserved0: u32,
    pub dbc: u32,
}

#[repr(C)]
pub struct HbaCmdTbl {
    pub cfis: [u8; 64],
    pub acmd: [u8; 16],
    reserved: [u8; 48],
    pub prdt_entry: [HbaPrdtEntry; 1],
}

#[repr(C)]
pub struct FisRegH2D {
    pub fis_type: u8,
    pub flags: u8,
    pub command: u8,
    pub featurel: u8,
    pub lba0: u8,
    pub lba1: u8,
    pub lba2: u8,
    pub device: u8,
    pub lba3: u8,
    pub lba4: u8,
    pub lba5: u8,
    pub featureh: u8,
    pub countl: u8,
    pub counth: u8,
    pub icc: u8,
    pub control: u8,
    reserved: [u8; 4],
}

#[repr(C)]
pub struct FisRegD2H {
    pub fis_type: u8,
    pub flags: u8,
    pub status: u8,
    pub error: u8,
    pub lba0: u8,
    pub lba1: u8,
    pub lba2: u8,
    pub device: u8,
    pub lba3: u8,
    pub lba4: u8,
    pub lba5: u8,
    reserved0: u8,
    pub countl: u8,
    pub counth: u8,
    reserved1: [u8; 2],
    reserved2: [u8; 4],
}

#[repr(C)]
pub struct FisData {
    pub fis_type: u8,
    pub flags: u8,
    reserved: [u8; 2],
    pub data: [u32; 1],
}

#[repr(C)]
pub struct FisPioSetup {
    pub fis_type: u8,
    pub flags: u8,
    pub status: u8,
    pub error: u8,
    pub lba0: u8,
    pub lba1: u8,
    pub lba2: u8,
    pub device: u8,
    pub lba3: u8,
    pub lba4: u8,
    pub lba5: u8,
    reserved0: u8,
    pub countl: u8,
    pub counth: u8,
    reserved1: u8,
    pub e_status: u8,
    pub tc: u16,
    reserved2: [u8; 2],
}

#[repr(C)]
pub struct FisDmaSetup {
    pub fis_type: u8,
    pub flags: u8,
    reserved: [u8; 2],
    pub dma_buffer_id: u64,
    reserved1: u32,
    pub dma_buffer_offset: u32,
    pub transfer_count: u32,
    reserved2: u32,
}

#[repr(C)]
pub struct HbaFis {
    pub dsfis: FisDmaSetup,
    padding0: [u8; 4],
    pub psfis: FisPioSetup,
    padding1: [u8; 12],
    pub rfis: FisRegD2H,
    padding2: [u8; 4],
    pub sdbfis: [u8; 8],
    pub ufis: [u8; 64],
    reserved: [u8; 96],
}

pub const HBA_PORT_CMD_ST: u32 = 0x0001;
pub const HBA_PORT_CMD_FRE: u32 = 0x0010;
pub const HBA_PORT_CMD_FR: u32 = 0x4000;
pub const HBA_PORT_CMD_CR: u32 = 0x8000;

pub const HBA_PX_IS_TFES: u32 = 1 << 30;

pub const HBA_PORT_SIG_ATA: u32 = 0x00000101;
pub const HBA_PORT_SIG_ATAPI: u32 = 0xEB140101;
pub const HBA_PORT_SIG_SEMB: u32 = 0xC33C0101;
pub const HBA_PORT_SIG_PM: u32 = 0x96690101;

pub const HBA_SSTS_DET_PRESENT: u32 = 0x3;
pub const HBA_SSTS_IPM_ACTIVE: u32 = 0x1;

pub const ATA_DEV_BUSY: u8 = 0x80;
pub const ATA_DEV_DRQ: u8 = 0x08;

pub const ATA_CMD_READ_DMA_EX: u8 = 0x25;
pub const ATA_CMD_WRITE_DMA_EX: u8 = 0x35;
pub const ATA_CMD_IDENTIFY: u8 = 0xEC;

pub const FIS_TYPE_REG_H2D: u8 = 0x27;
pub const FIS_TYPE_REG_D2H: u8 = 0x34;
pub const FIS_TYPE_DMA_ACT: u8 = 0x39;
pub const FIS_TYPE_DMA_SETUP: u8 = 0x41;
pub const FIS_TYPE_DATA: u8 = 0x46;
pub const FIS_TYPE_BIST: u8 = 0x58;
pub const FIS_TYPE_PIO_SETUP: u8 = 0x5F;
pub const FIS_TYPE_DEV_BITS: u8 = 0xA1;

pub const HBA_GHC_AE: u32 = 1 << 31;
pub const HBA_GHC_IE: u32 = 1 << 1;
pub const HBA_GHC_HR: u32 = 1 << 0;

pub const AHCI_CMD_HEADER_FLAGS_FIS_LEN: u16 = 5;
pub const AHCI_CMD_HEADER_FLAGS_WRITE: u16 = 1 << 6;
pub const AHCI_CMD_HEADER_FLAGS_PREFETCHABLE: u16 = 1 << 7;
pub const AHCI_CMD_HEADER_FLAGS_CLR_BUSY: u16 = 1 << 10;

pub const AHCI_PRDT_DBC_MASK: u32 = 0x3FFFFF;
pub const AHCI_PRDT_DBC_IPC: u32 = 1 << 31;

pub const AHCI_DEV_NULL: u8 = 0;
pub const AHCI_DEV_SATA: u8 = 1;
pub const AHCI_DEV_SATAPI: u8 = 2;
pub const AHCI_DEV_SEMB: u8 = 3;
pub const AHCI_DEV_PM: u8 = 4;

pub const HBA_PORT_DET_PRESENT: u32 = 0x3;
pub const HBA_PORT_IPM_ACTIVE: u32 = 0x1;