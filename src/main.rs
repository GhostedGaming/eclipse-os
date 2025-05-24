#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(eclipse_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
use alloc::format;

use bootloader::{BootInfo, entry_point};

use core::panic::PanicInfo;

use eclipse_os::task::{Task, executor::Executor, keyboard};
use eclipse_os::time;
use eclipse_os::vga_buffer::{self, Color, CursorStyle};
use eclipse_os::{print, println, port_println};
use eclipse_os::cpu::cpuid;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use eclipse_os::allocator;
    use eclipse_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    // Initialize memory management FIRST
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // Initialize heap IMMEDIATELY after memory setup
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

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
    print_status("Test Coms", test_port_print());

    #[cfg(test)]
    test_main();

    // Initialize and run the executor
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.spawn(Task::new(time::time_sync_task()));
    executor.run();
}

fn test_port_print() -> Result<(), ()> {
    port_println!("Test message from Eclipse OS!");

    Ok(())
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
fn trivial_assertion() -> Result<(), ()> {
    if 1 == 1 {
        Ok(())
    } else {
        Err(())
    }
}

/// Initiate time and return success status
fn initiate_time() -> Result<(), ()> {
    time::init(); // This now properly configures the PIT
    Ok(())
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    eclipse_os::hlt_loop();
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
    print_status(&format!("Async Number [{}]", number), if success { Ok(()) } else { Err(()) });
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
