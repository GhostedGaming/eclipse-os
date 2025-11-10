#![no_std]
#![no_main]

extern crate alloc;

use core::arch::asm;

// External crates
use alloc::{vec::Vec, vec};
use limine::BaseRevision;
use limine::request::{FramebufferRequest, MemoryMapRequest, RequestsEndMarker, RequestsStartMarker};

// Eclipse crates
use eclipse_framebuffer::{ ScrollingTextRenderer, println, print, panic_print};
use eclipse_ide::ide_init;
use eclipse_fs::{write_eclipse_fs, write_block, read_block, SuperBlock};
use eclipse_os::{gdt, idt, mem::mem};

static FONT: &[u8] = include_bytes!("../../eclipse_framebuffer/font/Mik_8x16.psf");

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
    println!("GDT Initialized");
    println!("Initializing IDT...");
    idt::idt_init();
    println!("IDT Initialized");
    asm!("sti");
    println!("Interrupts enabled");
    println!("Initializing IDE");
    ide_init(0, 0, 0, 0, 0);
    println!("IDE Initialized");
    
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
    
    println!("Writing to block 700");
    let test_data: Vec<u8> = vec![0x42; 512];
    println!("Writing 512 bytes (all 0x42) to block 700...");
    match write_block(0, &super_block, 700, &test_data) {
        Ok(()) => println!("Write successful!"),
        Err(e) => println!("Write failed: {:?}", e),
    }

    println!("Reading back block 700...");
    match read_block(0, &super_block, 700) {
        Ok(data) => {
            println!("Read {} bytes from block 700", data.len());

            let count_42 = data.iter().filter(|&&b| b == 0x42).count();
            let count_00 = data.iter().filter(|&&b| b == 0x00).count();

            println!("Expected all 0x42, got {} bytes of 0x42 and {} bytes of 0x00", count_42, count_00);

            print!("First 32 bytes: ");
            for i in 0..32.min(data.len()) {
                print!("{:02X} ", data[i]);
            }
            println!();
        }
        Err(e) => println!("Read failed: {:?}", e),
    }
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