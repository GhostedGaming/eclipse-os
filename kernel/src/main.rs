#![no_std]
#![no_main]

use core::arch::asm;

// External crates
use limine::BaseRevision;
use limine::request::{FramebufferRequest, MemoryMapRequest, RequestsEndMarker, RequestsStartMarker};

// Eclipse crates
use eclipse_framebuffer::{ ScrollingTextRenderer, println };
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

    println!("EclipseOS Starting...");
    
    println!("Initializing GDT...");
    gdt::gdt_init();
    println!("GDT Initialized");
    
    println!("Initializing IDT...");
    idt::idt_init();
    println!("IDT Initialized");
    
    println!("Initializing Memory Allocator...");
    if let Some(memmap_response) = MEMMAP_REQUEST.get_response() {
        mem::VMM::init(memmap_response);
        mem::init_allocator(memmap_response);
        println!("Memory Allocator Initialized");
    } else {
        println!("WARNING: No memory map available!");
    }

    println!("System Booted Successfully!");

    hcf();
}

#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    println!("KERNEL PANIC: {}", info);
    hcf();
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