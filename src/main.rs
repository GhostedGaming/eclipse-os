#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(eclipse_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use eclipse_os::{println,print};
use eclipse_os::task::{Task, executor::Executor, keyboard};
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use eclipse_os::vga_buffer::{self, Color};
use eclipse_os::shell::Shell;
use eclipse_os::time;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use eclipse_os::allocator;
    use eclipse_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    vga_buffer::set_color(Color::White, Color::Blue);
    println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");

    eclipse_os::init();

    vga_buffer::set_cursor_visibility(true);
    vga_buffer::set_cursor_style(vga_buffer::CursorStyle::Underline);
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // Initialize heap and print status
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
    print_status("allocater_init_heap");
    
    print_status("Panic_Handler");
    
    // Perform trivial assertion and print status
    print!("Performing trivial_assertion [");
    trivial_assertion();  // This will panic if it fails
    vga_buffer::set_color(Color::Green, Color::Blue);
    print!("OK");
    vga_buffer::set_color(Color::White, Color::Blue);
    print!("]\n");

    print!("initating time [");
    time::init();
    vga_buffer::set_color(Color::Green, Color::Blue);
    print!("OK");
    vga_buffer::set_color(Color::White, Color::Blue);
    print!("]\n");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.spawn(Task::new(time::time_sync_task()));
    executor.run();
}

/// Helper function to print status messages with consistent formatting
fn print_status(component: &str) {
    print!("{} [", component);
    vga_buffer::set_color(Color::Green, Color::Blue);
    print!("OK");
    vga_buffer::set_color(Color::White, Color::Blue);
    print!("]\n");
}

/// This function is called on panic.
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
    print!("async_number [");
    vga_buffer::set_color(Color::Green, Color::Blue);
    print!("{}", number);
    vga_buffer::set_color(Color::White, Color::Blue);
    print!("]\n");
    print_ascii();
}

fn print_ascii() {
    vga_buffer::set_color(Color::Purple, Color::Blue);
    vga_buffer::set_color(Color::Cyan, Color::Blue);
    println!("");
    println!("      --ECLIPSE OS--     ");
    println!("");
    vga_buffer::set_color(Color::White, Color::Blue);
    eclipse_os::task::keyboard::init_shell();
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

fn trivial_assertion() {
    assert_eq!(1, 1);
}
