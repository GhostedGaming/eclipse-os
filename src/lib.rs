#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
use core::panic::PanicInfo;

use interrupts::enable_apic;
use serial::info;
use x86_64::instructions::interrupts::enable;

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
pub mod vga_buffer;

pub fn init() {
    info("init: enabling interrupts\n");
    enable();
    info("init: interrupts::enable_apic()\n");
    enable_apic();
    info("init: interrupts::init_idt()\n");
    interrupts::init_idt();
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
        serial_println!("[ok]");
        info("Testable::run: finished test\n");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    info("test_runner: running tests\n");
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        info("test_runner: running a test\n");
        test.run();
    }
    info("test_runner: all tests done, exiting QEMU\n");
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(panic_info: &PanicInfo) -> ! {
    info("test_panic_handler: panic occurred\n");
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", panic_info);
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
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
    info("exit_qemu: write to port done\n");
}

pub fn hlt_loop() -> ! {
    info("hlt_loop: entering halt loop\n");
    loop {
        x86_64::instructions::hlt();
    }
}

// #[cfg(test)]
// use bootloader::{BootInfo, entry_point};

// #[cfg(test)]
// entry_point!(test_kernel_main);

/// Entry point for `cargo xtest`
// #[cfg(test)]
// fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
//     init();
//     test_main();
//     hlt_loop();
// }

#[cfg(test)]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    info("panic_handler: test panic\n");
    test_panic_handler(panic_info)
}
