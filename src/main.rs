#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(eclipse_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
use alloc::boxed::Box;
use alloc::format;

use alloc::string::ToString;
use eclipse_os::memory::BootInfoFrameAllocator;
use eclipse_os::serial::serial_write_str;
use spin::Mutex;
use uefi::boot::MemoryType;
use uefi::mem::memory_map::MemoryMapOwned;
use x86_64::structures::paging::{Mapper, Size4KiB};

use core::panic::PanicInfo;

use eclipse_os::cpu::cpuid;
use eclipse_os::task::{Task, executor::Executor, keyboard};
use eclipse_os::vga_buffer::{self, Color, CursorStyle};
use eclipse_os::{allocator, memory, serial, time};
use eclipse_os::{print, println};
use uefi::prelude::*;

#[entry]
fn efi_main() -> Status {
    if let Err(e) = uefi::helpers::init() {
        return e.status();
    }
    // Get the memory map from UEFI
    let memory_map = match uefi::boot::memory_map(MemoryType::LOADER_DATA) {
        Ok(map) => map,
        Err(e) => return e.status(),
    };

    // Construct BootInfo
    let boot_info = Box::leak(Box::new(BootInfo {
        memory_map: Mutex::new(memory_map),
        physical_memory_offset: eclipse_os::memory::PHYSICAL_MEMORY_OFFSET,
        _non_exhaustive: 0,
    }));

    kernel_main(boot_info)
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
    /// The offset into the virtual address space where the physical memory is mapped.
    ///
    /// Physical addresses can be converted to virtual addresses by adding this offset to them.
    ///
    /// The mapping of the physical memory allows to access arbitrary physical frames. Accessing
    /// frames that are also mapped at other virtual addresses can easily break memory safety and
    /// cause undefined behavior. Only frames reported as `USABLE` by the memory map in the `BootInfo`
    /// can be safely accessed.
    pub physical_memory_offset: u64,
    _non_exhaustive: u8, // `()` is not FFI safe
}

fn get_mapper(physical_memory_offset: u64) -> impl Mapper<Size4KiB> {
    // SAFETY: This is safe because we ensure that the physical memory is mapped correctly
    unsafe { memory::init(x86_64::VirtAddr::new(physical_memory_offset)) }
}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Lock the memory_map mutex to get access
    // let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&*memory_map_guard) };

    // Initialize heap IMMEDIATELY after memory setup
    // allocator::init_heap(&mut mapper, &mut frame_allocator)
    //     .expect("heap initialization failed");

    // Create a mapper
    // Create a frame allocator from the memory map
    let virt_addr = x86_64::VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(virt_addr) };
    let memory_map_guard = boot_info.memory_map.lock();
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&*memory_map_guard) };

    let _ = allocator::init_heap(&mut mapper, &mut frame_allocator);

    // Now it's safe to call other initialization
    eclipse_os::init();

    vga_buffer::set_cursor_style(CursorStyle::Underline);
    vga_buffer::set_color(Color::White, Color::Black);
    vga_buffer::set_cursor_visibility(true);

    // Initialize CPU info and print details
    cpuid::init_cpu_info();
    cpuid::print_cpu_info();

    print_status("Heap Initialization", Ok(()));
    print_status("Panic Handler Setup", Ok(()));
    print_status("Trivial Assertion", trivial_assertion());
    print_status("Time Initialization", initiate_time());
    // print_status("PC Speaker Initialization", init_pc_speaker_status());
    print_status("Test Coms", test_port_print());

    // Play startup sound after all initialization is complete
    play_startup_sound();

    #[cfg(test)]
    test_main();

    // Initialize and run the executor
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

fn test_port_print() -> Result<(), ()> {
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
    print!("{} [", component);

    match result {
        Ok(_) => {
            vga_buffer::set_color(Color::Green, Color::Black);
            print!("OK");
        }
        Err(_) => {
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
    print!("KERNEL PANIC: ");
    print!("{}", info.message());
    if let Some(location) = info.location() {
        print!(" at {}:{}", location.file(), location.line());
    }
    print!("\n");
    loop {}
}

pub fn print_panic_info_serial(info: &core::panic::PanicInfo) {
    use alloc::string::String;
    use core::fmt::Write;

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
    42
}

async fn example_task() {
    let number = async_number().await;
    let success = number == 42;
    print_status(
        &format!("Async Number [{}]", number),
        if success { Ok(()) } else { Err(()) },
    );

    print_ascii();
}

fn print_ascii() {
    vga_buffer::set_color(Color::Purple, Color::Black);
    println!("");
    println!("      --ECLIPSE OS--     ");
    println!("");
    vga_buffer::set_color(Color::White, Color::Black);
    eclipse_os::task::keyboard::init_shell();
}
