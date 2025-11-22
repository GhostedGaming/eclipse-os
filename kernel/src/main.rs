#![no_std]
#![no_main]

extern crate alloc;

use core::arch::asm;

// External crates

use limine::BaseRevision;
use limine::request::{FramebufferRequest, MemoryMapRequest, RequestsEndMarker, RequestsStartMarker};

// Eclipse crates
use eclipse_framebuffer::{ ScrollingTextRenderer, println, print, panic_print};
use ide::ide_init;
use eclipse_fs::{SuperBlock, write_eclipse_fs};
use eclipse_fs::file_ops::{create_file, read_file, delete_file};
use eclipse_fs::directory::DirectoryManager;
use eclipse_fs::inodes::InodeManager;
use ahci::find_ahci_controller;
use eclipse_pci::{check_all_buses, pci_find_ahci_controller, pci_enable_bus_master, pci_enable_memory_space};
use eclipse_threader::scheduler::scheduler::scheduler_init;
use eclipse_os::mem::mem::{VMM, VirtAddr, PhysAddr, PageTableEntry};
use eclipse_os::{gdt, idt, mem::mem};

static FONT: &[u8] = include_bytes!("../../eclipse_framebuffer/font/altc-8x16.psf");

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMMAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();
#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    assert!(BASE_REVISION.is_supported());
    let framebuffer_response = FRAMEBUFFER_REQUEST.get_response().expect("No framebuffer");
    let framebuffer = framebuffer_response.framebuffers().next().expect("No framebuffer available");
    ScrollingTextRenderer::init(
        framebuffer.addr(),
        framebuffer.width() as usize,
        framebuffer.height() as usize,
        framebuffer.pitch() as usize,
        framebuffer.bpp() as usize,
        FONT,
    );
    println!("Initializing Memory Allocator...");
    if let Some(memmap_response) = MEMMAP_REQUEST.get_response() {
        mem::VMM::init(memmap_response);
        mem::init_allocator(memmap_response);
        println!("Memory Allocator Initialized");
    } else {
        println!("WARNING: No memory map available!");
    }

    println!("EclipseOS Starting...");
    println!("Initializing GDT...");
    gdt::gdt_init();
    println!("Initializing IDT...");
    idt::idt_init();
    println!("IDT Initialized");
    asm!("sti");

    println!("Interrupts enabled");
    println!("Initializing IDE");
    ide_init(0, 0, 0, 0, 0);
    println!("IDE Initialized");
    
    println!("Initializing PCI");
    check_all_buses();
    println!("PCI scan complete");

    println!("Initializing AHCI");
    match pci_find_ahci_controller() {
        Some(ahci_dev) => {
            let abar_phys = ahci_dev.bar[5] as u64 & !0xF;
            println!("AHCI controller found at {}:{}:{}", ahci_dev.bus, ahci_dev.device, ahci_dev.function);
            println!("AHCI BAR5 (physical): 0x{:X}", abar_phys);

            if abar_phys == 0 {
                println!("Invalid AHCI BAR address");
            } else {
                pci_enable_bus_master(ahci_dev.bus, ahci_dev.device, ahci_dev.function);
                pci_enable_memory_space(ahci_dev.bus, ahci_dev.device, ahci_dev.function);

                let abar_virt = abar_phys | 0xFFFF800000000000;
                println!("AHCI ABAR (virtual): 0x{:X}", abar_virt);

                unsafe {
                    let pages_to_map = 4;
                    for i in 0..pages_to_map {
                        let offset = i * 0x1000;
                        let phys_addr = abar_phys + offset;
                        let virt_addr = abar_virt + offset;

                        match VMM::map_page(
                            VirtAddr::new(virt_addr),
                            PhysAddr::new(phys_addr),
                            PageTableEntry::PRESENT | 
                            PageTableEntry::WRITABLE | 
                            PageTableEntry::NO_CACHE
                        ) {
                            Some(()) => {},
                            None => {
                                println!("Failed to map AHCI page at offset 0x{:X}", offset);
                                hcf();
                            }
                        }
                    }
                }

                println!("AHCI ABAR mapped successfully");

                find_ahci_controller();
            }
        }
        None => {
            println!("No AHCI controller found");
        }
    }

    println!("Mapping APIC...");

    
    println!("Writing fs");
    write_eclipse_fs(0);
    
    println!("Reading superblock from disk...");
    let super_block: SuperBlock = match SuperBlock::read_super_block(0) {
        Ok(sb) => {
            println!("Superblock loaded: {}", sb);
            sb
        }
        Err(e) => {
            println!("Failed to read superblock: {}", e);
            hcf();
        }
    };
    
    println!("Loading bitmap from disk...");
    let bitmap = match eclipse_fs::BlockBitmap::from_disk(0, &super_block) {
        Ok(bm) => {
            println!("Bitmap loaded successfully");
            println!("Free blocks: {}", bm.free_blocks());
            println!("Used blocks: {}", bm.used_blocks());
            bm
        }
        Err(e) => {
            println!("Failed to load bitmap: {:?}", e);
            hcf();
        }
    };
    
    println!("\nInode Manager");
    match InodeManager::new(0, super_block, bitmap) {
        Ok(mut inode_manager) => {
            println!("Inode Manager initialized");
            
            println!("\nTesting File Creation");
            let test_data = b"Hello from EclipseOS!";
            match create_file(&mut inode_manager, test_data) {
                Ok(inode_idx) => {
                    println!("File created at inode {}", inode_idx);
                    
                    println!("\nTesting File Reading");
                    match read_file(&inode_manager, inode_idx) {
                        Ok(file_data) => {
                            println!("File read successfully: {} bytes", file_data.len());
                            print!("File content: ");
                            for &byte in file_data.iter() {
                                print!("{}", byte as char);
                            }
                            println!();
                        }
                        Err(e) => println!("Failed to read file: {:?}", e),
                    }
                    
                    println!("\nTesting Directory Creation");
                    match DirectoryManager::create_directory(&mut inode_manager) {
                        Ok(dir_inode) => {
                            println!("Directory created at inode {}", dir_inode);
                            
                            println!("\nTesting Directory Entry Addition");
                            match DirectoryManager::add_entry(&mut inode_manager, dir_inode, b"test_file.txt", inode_idx) {
                                Ok(()) => {
                                    println!("Entry added to directory");
                                    
                                    println!("\nTesting File Lookup");
                                    match DirectoryManager::find_entry(&inode_manager, dir_inode, b"test_file.txt") {
                                        Ok(Some(found_inode)) => {
                                            println!("Found file at inode {}", found_inode);
                                        }
                                        Ok(None) => println!("File not found in directory"),
                                        Err(e) => println!("Error searching directory: {:?}", e),
                                    }
                                    
                                    println!("\nTesting Directory Listing");
                                    match DirectoryManager::list_directory(&inode_manager, dir_inode) {
                                        Ok(entries) => {
                                            println!("Directory contains {} entries:", entries.len());
                                            for (inode, name) in entries {
                                                println!("  inode {}: {:?}", inode, core::str::from_utf8(&name).unwrap_or("invalid_utf8"));
                                            }
                                        }
                                        Err(e) => println!("Error listing directory: {:?}", e),
                                    }
                                }
                                Err(e) => println!("Failed to add entry: {:?}", e),
                            }
                        }
                        Err(e) => println!("Failed to create directory: {:?}", e),
                    }
                    
                    println!("\nTesting File Deletion");
                    match delete_file(&mut inode_manager, inode_idx) {
                        Ok(()) => println!("File deleted successfully"),
                        Err(e) => println!("Failed to delete file: {:?}", e),
                    }
                }
                Err(e) => println!("Failed to create file: {:?}", e),
            }
        }
        Err(e) => println!("Failed to initialize inode manager: {:?}", e),
    }
    
    println!("\nFilesystem Tests Complete");

    println!("Initializing Scheduler...");
    scheduler_init();
    println!("Scheduler Initialized");

    hcf();
}

#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    let (rax, rbx, rcx, rdx): (u64, u64, u64, u64);
    let (rsi, rdi, rbp, rsp): (u64, u64, u64, u64);
    let (r8, r9, r10, r11): (u64, u64, u64, u64);
    let (r12, r13, r14, r15): (u64, u64, u64, u64);
    let (rflags, cs, ss): (u64, u16, u16);
    
    unsafe {
        asm!(
            "mov {}, rax",
            "mov {}, rbx",
            "mov {}, rcx",
            "mov {}, rdx",
            out(reg) rax,
            out(reg) rbx,
            out(reg) rcx,
            out(reg) rdx,
        );
        
        asm!(
            "mov {}, rsi",
            "mov {}, rdi",
            "mov {}, rbp",
            "mov {}, rsp",
            out(reg) rsi,
            out(reg) rdi,
            out(reg) rbp,
            out(reg) rsp,
        );
        
        asm!(
            "mov {}, r8",
            "mov {}, r9",
            "mov {}, r10",
            "mov {}, r11",
            out(reg) r8,
            out(reg) r9,
            out(reg) r10,
            out(reg) r11,
        );
        
        asm!(
            "mov {}, r12",
            "mov {}, r13",
            "mov {}, r14",
            "mov {}, r15",
            out(reg) r12,
            out(reg) r13,
            out(reg) r14,
            out(reg) r15,
        );
        
        asm!("pushfq", "pop {}", out(reg) rflags);
        asm!("mov {:x}, cs", out(reg) cs);
        asm!("mov {:x}, ss", out(reg) ss);
    }
    
    panic_print!(
        "KERNEL PANIC\n{}\n\n\
        Register Dump:\n\
        RAX: 0x{:016x}  RBX: 0x{:016x}  RCX: 0x{:016x}  RDX: 0x{:016x}\n\
        RSI: 0x{:016x}  RDI: 0x{:016x}  RBP: 0x{:016x}  RSP: 0x{:016x}\n\
        R8:  0x{:016x}  R9:  0x{:016x}  R10: 0x{:016x}  R11: 0x{:016x}\n\
        R12: 0x{:016x}  R13: 0x{:016x}  R14: 0x{:016x}  R15: 0x{:016x}\n\
        RFLAGS: 0x{:016x}\n\
        CS:  0x{:04x}      SS:  0x{:04x}",
        info,
        rax, rbx, rcx, rdx,
        rsi, rdi, rbp, rsp,
        r8, r9, r10, r11,
        r12, r13, r14, r15,
        rflags,
        cs, ss
    );
    
    loop {
        unsafe {
            asm!(
                "cli",
                "hlt"
            );
        }
    }
}

fn hcf() -> ! {
    loop {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            asm!("hlt");
            #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
            asm!("wfi");
            #[cfg(target_arch = "loongarch64")]
            asm!("idle 0");
        }
    }
}