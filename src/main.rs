#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(eclipse_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use eclipse_os::{println, print};
use eclipse_os::task::{Task, executor::Executor, keyboard};
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use eclipse_os::vga_buffer::{self, Color};
use eclipse_os::time;
use alloc::format;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use eclipse_os::allocator;
    use eclipse_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    eclipse_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // Initialize heap and print status
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap initialization failed");
    print_status("allocator_init_heap");

    // Print status for Panic Handler
    print_status("Panic_Handler");

    // Perform trivial assertion and print status
    trivial_assertion();

    // Initialize time and print status
    initiate_time();

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
    vga_buffer::set_color(Color::Green, Color::Black);
    print!("OK");
    vga_buffer::set_color(Color::White, Color::Black);
    print!("]\n");
}

/// Perform trivial assertion and print status
fn trivial_assertion() {
    assert_eq!(1, 1);
    print_status("trivial_assertion");
}

/// Initiate time and print status
fn initiate_time() {
    time::init();
    print_status("time_init");
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
    print_status(&format!("async_number [{}]", number));
    print_ascii();
}

fn print_ascii() {
    vga_buffer::set_color(Color::Purple, Color::Black);
    vga_buffer::set_color(Color::Cyan, Color::Black);
    println!("");
    println!("      --ECLIPSE OS--     ");
    println!("");
    vga_buffer::set_color(Color::White, Color::Black);
    eclipse_os::task::keyboard::init_shell();
}
