#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]

extern crate alloc;

use core::panic::PanicInfo;
use serial::info;
use spin::{Mutex, Once};
use uefi::mem::memory_map::MemoryMapOwned;
use uefi::proto::console::text::Output;

pub mod cpu;
pub mod crude_storage;
pub mod fs;
pub mod gdt;
pub mod intereperter;
pub mod interrupts;
pub mod memory;
pub mod pc_speaker;
pub mod rtc;
pub mod serial;
pub mod shell;
pub mod shutdown;
pub mod task;
pub mod text_editor;
pub mod time;
pub mod uefi_text_buffer;
pub mod vga_buffer;
pub mod wifi;

// Make text_output globally accessible
pub static TEXT_OUTPUT: Once<Mutex<OutputForced>> = Once::new();

pub struct OutputForced(pub *mut Output);
unsafe impl Send for OutputForced {}

#[derive(Debug)]
#[repr(C)]
pub struct BootInfo {
    /// A map of the physical memory regions of the underlying machine.
    pub memory_map: Once<Mutex<MemoryMapOwned>>,
    /// Prevent external construction and ensure FFI compatibility
    pub _non_exhaustive: u8,
}

pub fn init() {
    info("init: initializing GDT\n");
    gdt::init();
    
    info("init: initializing IDT\n");
    interrupts::init_idt();
    
    info("init: initializing PICs\n");
    unsafe { 
        interrupts::PICS.lock().initialize();
    }
    
    info("init: enabling interrupts\n");
    // Enable interrupts using inline assembly instead of x86_64 crate
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack));
    }
    
    info("init: done\n");
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        info("Testable::run: starting test\n");
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_print!("[ok]");
        info("Testable::run: finished test\n");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    info("test_runner: running tests\n");
    serial_print!("Running {} tests", tests.len());
    for test in tests {
        info("test_runner: running a test\n");
        test.run();
    }
    info("test_runner: all tests done, exiting QEMU\n");
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(panic_info: &PanicInfo) -> ! {
    info("test_panic_handler: panic occurred\n");
    serial_print!("[failed]\n");
    serial_print!("Error: {}\n", panic_info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    info("exit_qemu: exiting QEMU\n");
    unsafe {
        // Direct port I/O instead of using x86_64 crate
        core::arch::asm!(
            "out dx, eax",
            in("dx") 0xf4u16,
            in("eax") exit_code as u32,
            options(nomem, nostack, preserves_flags)
        );
    }
    info("exit_qemu: write to port done\n");
}

pub fn hlt_loop() -> ! {
    info("hlt_loop: entering halt loop\n");
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}

// Re-export commonly used items for convenience
pub use interrupts::PICS;

#[cfg(test)]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    info("panic_handler: test panic\n");
    test_panic_handler(panic_info)
}
