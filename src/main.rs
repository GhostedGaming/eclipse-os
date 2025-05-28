#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(eclipse_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
use alloc::format;

use alloc::string::ToString;
use eclipse_os::serial::serial_write_str;
use spin::Mutex;
use uefi::boot::MemoryType;
use uefi::mem::memory_map::MemoryMapOwned;

use core::panic::PanicInfo;

use eclipse_os::cpu::cpuid;
use eclipse_os::task::{Task, executor::Executor, keyboard};
use eclipse_os::vga_buffer::{self, Color, CursorStyle};
use eclipse_os::{serial, time};
use eclipse_os::{print, println};
use uefi::prelude::*;

mod bump_allocator;
use bump_allocator::BumpAllocator;

const HEAP_SIZE: usize = 4096;

#[global_allocator]
static GLOBAL: BumpAllocator<HEAP_SIZE> = BumpAllocator::new();

#[entry]
fn efi_main() -> Status {
    serial::info("efi_main: Entered UEFI entry point\n");
    if let Err(e) = uefi::helpers::init() {
        serial::info("efi_main: UEFI helpers init failed\n");
        return e.status();
    }
    serial::info("efi_main: UEFI helpers initialized\n");
    // Get the memory map from UEFI
    let memory_map = match uefi::boot::memory_map(MemoryType::LOADER_DATA) {
        Ok(map) => {
            serial::info("efi_main: Got UEFI memory map\n");
            map
        }
        Err(e) => {
            serial::info("efi_main: Failed to get UEFI memory map\n");
            return e.status();
        }
    };

    // Construct BootInfo on the stack (no heap allocation)
    serial::info("efi_main: Constructing BootInfo\n");
    let mut boot_info = BootInfo {
        memory_map: Mutex::new(memory_map),
        _non_exhaustive: 0,
    };

    serial::info("efi_main: Using bump allocator for heap initialization\n");
    serial::info("efi_main: Calling kernel_main\n");
    kernel_main(&mut boot_info)
}

#[derive(Debug)]
#[repr(C)]
pub struct BootInfo {
    /// A map of the physical memory regions of the underlying machine.
    ///
    /// The bootloader queries this information from the BIOS/UEFI firmware and translates this
    /// information to Rust types. It also marks any memory regions that the bootloader uses in
    /// the memory map before passing it to the kernel. Regions marked as usable can be freely
    /// used by the kernel.
    pub memory_map: Mutex<MemoryMapOwned>,
    _non_exhaustive: u8, // `()` is not FFI safe
}

fn kernel_main(boot_info: &mut BootInfo) -> ! {
    serial::info("kernel_main: Entered kernel_main\n");
    serial::info("kernel_main: Using bump allocator for heap allocations\n");
    // Create a mapper
    // Create a frame allocator from the memory map
    serial::info("kernel_main: Initializing memory mapper\n");
    serial::info("kernel_main: Locking memory_map mutex\n");
    serial::info("kernel_main: Initializing frame allocator\n");

    serial::info("kernel_main: Calling eclipse_os::init()\n");
    eclipse_os::init();

    serial::info("kernel_main: Setting VGA cursor style\n");
    vga_buffer::set_cursor_style(CursorStyle::Underline);
    vga_buffer::set_color(Color::White, Color::Black);
    vga_buffer::set_cursor_visibility(true);

    serial::info("kernel_main: Initializing CPU info\n");
    cpuid::init_cpu_info();
    cpuid::print_cpu_info();

    print_status("Heap Initialization", Ok(()));
    print_status("Panic Handler Setup", Ok(()));
    print_status("Trivial Assertion", trivial_assertion());
    print_status("Time Initialization", initiate_time());
    // print_status("PC Speaker Initialization", init_pc_speaker_status());
    print_status("Test Coms", test_port_print());

    serial::info("kernel_main: Playing startup sound\n");
    play_startup_sound();

    #[cfg(test)]
    test_main();

    serial::info("kernel_main: Initializing executor\n");
    // Initialize and run the executor
    let mut executor = Executor::new();
    serial::info("kernel_main: Spawning example_task\n");
    executor.spawn(Task::new(example_task()));
    serial::info("kernel_main: Spawning keyboard::print_keypresses\n");
    executor.spawn(Task::new(keyboard::print_keypresses()));
    serial::info("kernel_main: Running executor\n");
    executor.run();
}

fn test_port_print() -> Result<(), ()> {
    serial::info("test_port_print: Sending test string to serial\n");
    serial::info("Hello");
    Ok(())
}

/// Initialize PC Speaker and return status
fn init_pc_speaker_status() -> Result<(), ()> {
    // init_pc_speaker();
    Ok(())
}

/// Play the startup sound
fn play_startup_sound() {
    // Play the startup melody
    // play_melody(Melody::PowerOn);
}

/// Helper function to print status messages with consistent formatting
fn print_status(component: &str, result: Result<(), ()>) {
    serial::info(&format!("print_status: {} ...\n", component));
    print!("{} [", component);

    match result {
        Ok(_) => {
            serial::info("print_status: OK\n");
            vga_buffer::set_color(Color::Green, Color::Black);
            print!("OK");
        }
        Err(_) => {
            serial::info("print_status: FAIL\n");
            vga_buffer::set_color(Color::Red, Color::Black);
            print!("FAIL");
        }
    }

    vga_buffer::set_color(Color::White, Color::Black);
    print!("]\n");
}

/// Perform trivial assertion and return success status
#[allow(clippy::eq_op)]
fn trivial_assertion() -> Result<(), ()> {
    if 1 == 1 { Ok(()) } else { Err(()) }
}

/// Initiate time and return success status
fn initiate_time() -> Result<(), ()> {
    time::init(); // This now properly configures the PIT
    Ok(())
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial::info("panic: Kernel panic occurred!\n");
    print!("KERNEL PANIC: ");
    print!("{}", info.message());
    if let Some(location) = info.location() {
        print!(" at {}:{}", location.file(), location.line());
        serial::info(&format!(
            "panic: at {}:{}\n",
            location.file(),
            location.line()
        ));
    }
    print!("\n");
    print_panic_info_serial(info);
    loop {}
}

pub fn print_panic_info_serial(info: &core::panic::PanicInfo) {
    use alloc::string::String;
    use core::fmt::Write;

    serial::info("print_panic_info_serial: Printing panic info to serial\n");

    // Print a clear panic header
    serial_write_str("\n=== KERNEL PANIC ===\n");

    // Print location if available
    if let Some(location) = info.location() {
        serial_write_str("Location: ");
        serial_write_str(location.file());
        serial_write_str(":");
        serial_write_str(&location.line().to_string());
        serial_write_str(":");
        serial_write_str(&location.column().to_string());
        serial_write_str("\n");
    } else {
        serial_write_str("Location: <unknown>\n");
    }

    // Print panic message (payload)
    serial_write_str("Message: ");
    let mut msg_buf = String::new();
    let args = info.message();
    let _ = write!(&mut msg_buf, "{args}");
    serial_write_str(&msg_buf);
    serial_write_str("\n");

    serial_write_str("====================\n\n");
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    eclipse_os::test_panic_handler(info);
}

async fn async_number() -> u32 {
    serial::info("async_number: returning 42\n");
    42
}

async fn example_task() {
    serial::info("example_task: started\n");
    let number = async_number().await;
    serial::info(&format!("example_task: async_number returned {}\n", number));
    let success = number == 42;
    print_status(
        &format!("Async Number [{}]", number),
        if success { Ok(()) } else { Err(()) },
    );

    print_ascii();
}

fn print_ascii() {
    serial::info("print_ascii: Printing ASCII art and initializing shell\n");
    vga_buffer::set_color(Color::Purple, Color::Black);
    println!("");
    println!("      --ECLIPSE OS--     ");
    println!("");
    vga_buffer::set_color(Color::White, Color::Black);
    eclipse_os::task::keyboard::init_shell();
}
