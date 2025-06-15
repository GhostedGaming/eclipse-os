#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(eclipse_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]

extern crate alloc;
use alloc::format;

use alloc::string::ToString;
use eclipse_os::serial::{info, serial_write_str};
use spin::Mutex;
use uefi::boot::{get_handle_for_protocol, open_protocol_exclusive};
use uefi::proto::console::text::Output;

use eclipse_os::OutputForced;
use eclipse_os::time;
use eclipse_os::uefi_text_buffer::print_message;
// use eclipse_os::vga_buffer::{self, Color};
use eclipse_os::{TEXT_OUTPUT};
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

    if let Err(e) = uefi::helpers::init() {
        info("efi_main: UEFI helpers init failed\n");
        return e.status();
    }
    info("efi_main: UEFI helpers initialized\n");

    let handle = get_handle_for_protocol::<Output>().unwrap();
    let mut output = open_protocol_exclusive::<Output>(handle).unwrap();

    let raw_output = OutputForced(&mut *output as *mut Output);
    TEXT_OUTPUT.call_once(|| Mutex::new(raw_output));

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

    print_message("Eclipse OS Booting...\n");

    // Show time immediately after UEFI init
    show_current_time();

    info("efi_main: Using bump allocator for heap initialization\n");
    info("efi_main: Calling kernel_main\n");

    kernel_main()
}

fn show_current_time() {
    let utc_time = time::get_current_time();
    let local_time = time::get_current_time_local();
    let tz_offset = time::get_timezone_offset();
    
    let utc_str = format!("UTC Time: {}", utc_time);
    print_message(&utc_str);
    
    let local_str = format!("Local Time: {} (UTC{}{})", 
        local_time,
        if tz_offset >= 0 { "+" } else { "" },
        tz_offset
    );
    print_message(&local_str);
    
    info(&format!("UTC: {}, Local: {}, TZ: {}\n", utc_time, local_time, tz_offset));
}

fn kernel_main() -> ! {
    info("kernel_main: Entered kernel_main\n");
    info("kernel_main: Using bump allocator for heap allocations\n");

    print_message("Eclipse OS - Initializing...\n");

    // Show time before eclipse_os::init()
    show_current_time();

    info("kernel_main: Calling eclipse_os::init()\n");
    eclipse_os::init();

    // Show time after eclipse_os::init()
    show_current_time();

    info("kernel_main: Initializing time system\n");
    print_message("Initializing time system...\n");
    
    match initiate_time() {
        Ok(_) => {
            info("kernel_main: Time system initialized successfully\n");
            print_message("Time System: [OK]\n");
        }
        Err(_) => {
            info("kernel_main: Time system initialization failed\n");
            print_message("Time System: [FAIL]\n");
        }
    }

    // Show time after time system init
    show_current_time();

    // Wait a moment for timer to start
    for _ in 0..1000000 {
        core::hint::spin_loop();
    }

    display_time_info();

    print_message("Hello from Eclipse OS!\n");

    print_message("System Ready!\n");

    // Show time info again after system is ready
    display_time_info();

    // Test time functions to verify they work
    test_time_functions();

    let mut counter = 0u64;
    
    loop {
        // Force display time info every 50 million iterations
        if counter % 50000000 == 0 {
            let loop_msg = format!("=== Loop iteration {} ===", counter);
            print_message(&loop_msg);
            
            // Always show current RTC time
            show_current_time();
            
            // Show timer status
            let current_ticks = time::get_ticks();
            let timer_msg = format!("Timer ticks: {}", current_ticks);
            print_message(&timer_msg);
            
            // Show uptime if timer is working
            let uptime_seconds = time::get_uptime_seconds();
            if uptime_seconds > 0 {
                let uptime_str = time::get_uptime_string();
                let uptime_msg = format!("Uptime: {}", uptime_str);
                print_message(&uptime_msg);
            } else {
                print_message("Timer not running - no uptime available");
            }
            
            print_message(""); // Empty line
        }
        
        counter += 1;
        core::hint::spin_loop();
    }
}

fn display_time_info() {
    info("display_time_info: Displaying time information\n");
    print_message("=== TIME INFORMATION ===");
    
    // Always show current RTC time
    show_current_time();

    // Show boot time if available
    if let Some(boot_time_utc) = time::get_boot_time() {
        if let Some(boot_time_local) = time::get_boot_time_local() {
            let boot_str = format!("Boot Time UTC: {}", boot_time_utc);
            print_message(&boot_str);
            let boot_local_str = format!("Boot Time Local: {}", boot_time_local);
            print_message(&boot_local_str);
            info(&format!("Boot time UTC: {}, Local: {}\n", boot_time_utc, boot_time_local));
        }
    } else {
        print_message("Boot Time: Not available");
    }

    // Show timer information
    let ticks = time::get_ticks();
    let ticks_str = format!("Timer Ticks: {}", ticks);
    print_message(&ticks_str);
    info(&format!("Timer ticks: {}\n", ticks));

    if ticks > 0 {
        // Timer is working, show calculated times
        let uptime_seconds = time::get_uptime_seconds();
        let uptime_str = time::get_uptime_string();
        let uptime_display = format!("Uptime: {} ({}s)", uptime_str, uptime_seconds);
        print_message(&uptime_display);
        info(&format!("System uptime: {} seconds\n", uptime_seconds));

        let time_ms = time::get_time_ms();
        let time_ns = time::get_time_ns();
        let precision_str = format!("Precision: {}ms / {}ns", time_ms, time_ns);
        print_message(&precision_str);
        info(&format!("Time precision: {}ms, {}ns\n", time_ms, time_ns));
    } else {
        print_message("Timer not running - uptime calculations unavailable");
    }

    // Show CPU frequency if available
    if let Some(cpu_freq) = time::get_cpu_frequency_hz() {
        let freq_str = format!("CPU Frequency: {} Hz", cpu_freq);
        print_message(&freq_str);
        info(&format!("CPU frequency: {} Hz\n", cpu_freq));
        
        if let Some(precise_ns) = time::get_precise_time_ns() {
            let precise_str = format!("Precise Time: {}ns", precise_ns);
            print_message(&precise_str);
            info(&format!("Precise time: {}ns\n", precise_ns));
        }
    } else {
        print_message("CPU frequency not available");
    }

    print_message("========================");
    print_message(""); // Empty line
}

fn display_uptime_info() {
    show_current_time();
    
    let ticks = time::get_ticks();
    if ticks > 0 {
        let uptime_str = time::get_uptime_string();
        let status = format!("Uptime: {}", uptime_str);
        print_message(&status);
        info(&format!("Status update - {}\n", status));
    } else {
        print_message("Timer not running");
    }
}

fn test_time_functions() {
    info("test_time_functions: Testing time functions\n");
    print_message("=== TESTING TIME FUNCTIONS ===");

    // Test 1: Show current time multiple times
    print_message("Test 1: Multiple RTC reads");
    for i in 1..=3 {
        let test_msg = format!("RTC Read #{}", i);
        print_message(&test_msg);
        show_current_time();
        
        // Manual delay using CPU cycles
        for _ in 0..10000000 {
            core::hint::spin_loop();
        }
    }

    // Test 2: Timer tick test
    print_message("Test 2: Timer tick test");
    let start_ticks = time::get_ticks();
    print_message(&format!("Start ticks: {}", start_ticks));
    
    // Wait and check again
    for _ in 0..50000000 {
        core::hint::spin_loop();
    }
    
    let end_ticks = time::get_ticks();
    print_message(&format!("End ticks: {}", end_ticks));
    
    if end_ticks > start_ticks {
        print_message("Timer is working!");
    } else {
        print_message("Timer is NOT working - interrupts may not be enabled");
    }

    // Test 3: Performance counter
    print_message("Test 3: Performance counter");
    let counter = time::PerformanceCounter::new();
    
    for _ in 0..1000000 {
        core::hint::spin_loop();
    }
    
    let elapsed_ns = counter.elapsed_ns();
    let elapsed_us = counter.elapsed_us();
    let elapsed_ms = counter.elapsed_ms();
    
    let perf_result = format!("Performance: {}ns / {}us / {}ms", elapsed_ns, elapsed_us, elapsed_ms);
    print_message(&perf_result);
    info(&format!("Performance test: {}ns, {}us, {}ms\n", elapsed_ns, elapsed_us, elapsed_ms));

    print_message("=== TIME FUNCTION TESTS COMPLETED ===");
    print_message("");
}

fn manual_delay(ms: u32) {
    info(&format!("Manual delay: {}ms\n", ms));
    time::delay_ms(ms as u64);
}

fn print_status(component: &str, result: Result<(), ()>) {
    info(&format!("print_status: {component} ...\n"));
    print_message(&format!("{} [", component));

    match result {
        Ok(_) => {
            info("print_status: OK\n");
            print_message("OK");
        }
        Err(_) => {
            info("print_status: FAIL\n");
            print_message("FAIL");
        }
    }

    print_message("]\n");
}

#[allow(clippy::eq_op)]
fn trivial_assertion() -> Result<(), ()> {
    if 1 == 1 { Ok(()) } else { Err(()) }
}

fn initiate_time() -> Result<(), ()> {
    time::init();
    Ok(())
}

#[cfg(not(test))]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    info("panic: Kernel panic occurred!\n");
    
    print_message("KERNEL PANIC: ");
    print_message(&format!("{}", panic_info.message()));
    if let Some(location) = panic_info.location() {
        print_message(&format!(" at {}:{}", location.file(), location.line()));
        info(&format!(
            "panic: at {}:{}\n",
            location.file(),
            location.line()
        ));
    }
    print_message("\n");
    print_panic_info_serial(panic_info);
    loop {}
}

pub fn print_panic_info_serial(panic_info: &core::panic::PanicInfo) {
    use alloc::string::String;
    use core::fmt::Write;

    info("print_panic_info_serial: Printing panic info to serial\n");

    serial_write_str("\n=== KERNEL PANIC ===\n");

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
    print_message("");
    print_message("      --ECLIPSE OS--     ");
    print_message("");
    
    eclipse_os::task::keyboard::init_shell();
}

pub fn get_text_output() -> &'static Mutex<OutputForced> {
    TEXT_OUTPUT.get().expect("TEXT_OUTPUT not initialized")
}
