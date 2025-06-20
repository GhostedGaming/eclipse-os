#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

pub mod allocator;
pub mod cpu;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod task;
pub mod vga_buffer;
pub mod shell;
pub mod time;
pub mod shutdown;
pub mod fs;
pub mod pc_speaker;
pub mod text_editor;
pub mod intereperter;
pub mod coms;
pub mod rtc;
pub mod crude_storage;

pub fn init() {
    cpu::gdt::init();
    interrupts::init_idt();
    unsafe { 
        interrupts::PICS.lock().initialize();
    }
    
    // Enable interrupts using inline assembly
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack));
    }
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        // Direct port I/O for QEMU exit
        core::arch::asm!(
            "out dx, eax",
            in("dx") 0xf4u16,
            in("eax") exit_code as u32,
            options(nomem, nostack, preserves_flags)
        );
    }
}

pub fn hlt_loop() -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}

// Re-export commonly used items for convenience
pub use interrupts::PICS;
pub use cpu::gdt;

#[cfg(test)]
use bootloader::{BootInfo, entry_point};

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo xtest`
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

// Test cases
#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_init_sequence() {
        // Test that initialization doesn't panic
        // This will be called after init() in test_kernel_main
    }
}