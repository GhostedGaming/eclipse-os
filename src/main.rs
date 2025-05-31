#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(eclipse_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]

extern crate alloc;
use alloc::format;

use alloc::string::ToString;
use eclipse_os::pc_speaker::init_pc_speaker;
use eclipse_os::serial::{info, serial_write_str};
use spin::Mutex;
use uefi::boot::{get_handle_for_protocol, open_protocol_exclusive};
use uefi::proto::console::text::Output;

use eclipse_os::OutputForced;
use eclipse_os::time;
use eclipse_os::uefi_text_buffer::print_message;
use eclipse_os::vga_buffer::{self, Color};
use eclipse_os::{TEXT_OUTPUT, print, println};
use uefi::prelude::*;

use core::panic::PanicInfo;

mod bump_allocator;
use bump_allocator::BumpAllocator;

const HEAP_SIZE: usize = 4096;

#[global_allocator]
static GLOBAL: BumpAllocator<HEAP_SIZE> = BumpAllocator::new();

#[entry]
fn efi_main() -> Status {
    info("efi_main: Entered UEFI entry point\n");

    // Initialize UEFI helpers
    if let Err(e) = uefi::helpers::init() {
        info("efi_main: UEFI helpers init failed\n");
        return e.status();
    }
    info("efi_main: UEFI helpers initialized\n");

    // Get the UEFI memory map
    // let memory_map = match uefi::boot::memory_map(MemoryType::LOADER_DATA) {
    //     Ok(map) => {
    //         info("efi_main: Got UEFI memory map\n");
    //         map
    //     }
    //     Err(e) => {
    //         info("efi_main: Failed to get UEFI memory map\n");
    //         return e.status();
    //     }
    // };

    // Initialize the text output protocol and store it
    let handle = get_handle_for_protocol::<Output>().unwrap();
    let mut output = open_protocol_exclusive::<Output>(handle).unwrap();

    // Store Output in global Once
    let raw_output = OutputForced(&mut *output as *mut Output);
    TEXT_OUTPUT.call_once(|| Mutex::new(raw_output));

    // Re-lock our text output mutex for now
    if let Some(mutex) = TEXT_OUTPUT.get() {
        let output = mutex.lock();
        unsafe {
            if let Some(out) = output.0.as_mut() {
                out.reset(false).expect("Failed to reset output!");
            } else {
                info("Failed to get mutable Output for reset!\n");
            }
        }
    }

    // **Use print_message to display boot text**
    print_message("Eclipse OS Booting...\n");

    info("efi_main: Using bump allocator for heap initialization\n");
    info("efi_main: Calling kernel_main\n");

    // Pass BootInfo to kernel_main
    kernel_main()
}

fn kernel_main() -> ! {
    info("kernel_main: Entered kernel_main\n");
    info("kernel_main: Using bump allocator for heap allocations\n");

    info("kernel_main: Calling eclipse_os::init()\n");
    eclipse_os::init();

    // info("kernel_main: Setting VGA cursor style\n");
    // vga_buffer::set_cursor_style(CursorStyle::Underline);
    // vga_buffer::set_color(Color::White, Color::Black);
    // vga_buffer::set_cursor_visibility(true);

    // info("kernel_main: Initializing CPU info\n");
    // cpuid::init_cpu_info();
    // cpuid::print_cpu_info();

    // print_status("Heap Initialization", Ok(()));
    // print_status("Panic Handler Setup", Ok(()));
    // print_status("Trivial Assertion", trivial_assertion());
    // print_status("Time Initialization", initiate_time());
    // // print_status("PC Speaker Initialization", init_pc_speaker_status());
    // print_status("Test Coms", test_port_print());

    // info("kernel_main: Playing startup sound\n");
    // play_startup_sound();

    // #[cfg(test)]
    // test_main();

    // Never exit

    print_message("Hello from eclipse OS!");

    info("Initilizing pc speaker\n");

    init_pc_speaker_status().expect("Couldnt init speaker!");

    play_startup_sound();

    //info("kernel_main: Initializing executor\n");
    //// Initialize and run the executor
    //let mut executor = Executor::new();
    //info("kernel_main: Spawning example_task\n");
    //executor.spawn(Task::new(example_task()));
    //info("kernel_main: Spawning keyboard::print_keypresses\n");
    //executor.spawn(Task::new(keyboard::print_keypresses()));
    //info("kernel_main: Running executor\n");
    //executor.run();

    loop {
        core::hint::spin_loop();
    }
}

fn test_port_print() -> Result<(), ()> {
    info("test_port_print: Sending test string to serial\n");
    info("Hello");
    Ok(())
}

/// Initialize PC Speaker and return status
fn init_pc_speaker_status() -> Result<(), ()> {
    init_pc_speaker();
    Ok(())
}

/// Play the startup sound
fn play_startup_sound() {
    // Play the startup melody
    eclipse_os::pc_speaker::play_melody(eclipse_os::pc_speaker::Melody::PowerOn);
}

/// Helper function to print status messages with consistent formatting
fn print_status(component: &str, result: Result<(), ()>) {
    info(&format!("print_status: {component} ...\n"));
    print!("{} [", component);

    match result {
        Ok(_) => {
            info("print_status: OK\n");
            vga_buffer::set_color(Color::Green, Color::Black);
            print!("OK");
        }
        Err(_) => {
            info("print_status: FAIL\n");
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
fn panic(panic_info: &PanicInfo) -> ! {
    info("panic: Kernel panic occurred!\n");
    print!("KERNEL PANIC: ");
    print!("{}", panic_info.message());
    if let Some(location) = panic_info.location() {
        print!(" at {}:{}", location.file(), location.line());
        info(&format!(
            "panic: at {}:{}\n",
            location.file(),
            location.line()
        ));
    }
    print!("\n");
    print_panic_info_serial(panic_info);
    loop {}
}

pub fn print_panic_info_serial(panic_info: &core::panic::PanicInfo) {
    use alloc::string::String;
    use core::fmt::Write;

    info("print_panic_info_serial: Printing panic info to serial\n");

    // Print a clear panic header
    serial_write_str("\n=== KERNEL PANIC ===\n");

    // Print location if available
    if let Some(location) = panic_info.location() {
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
    let args = panic_info.message();
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
    info("async_number: returning 42\n");
    42
}

async fn example_task() {
    info("example_task: started\n");
    let number = async_number().await;
    info(&format!("example_task: async_number returned {number}\n"));
    let success = number == 42;
    print_status(
        &format!("Async Number [{number}]"),
        if success { Ok(()) } else { Err(()) },
    );

    print_ascii();
}

fn print_ascii() {
    info("print_ascii: Printing ASCII art and initializing shell\n");
    vga_buffer::set_color(Color::Purple, Color::Black);
    println!("");
    println!("      --ECLIPSE OS--     ");
    println!("");
    vga_buffer::set_color(Color::White, Color::Black);
    eclipse_os::task::keyboard::init_shell();
}

// Public getter for text_output
pub fn get_text_output() -> &'static Mutex<OutputForced> {
    TEXT_OUTPUT.get().expect("TEXT_OUTPUT not initialized")
}
