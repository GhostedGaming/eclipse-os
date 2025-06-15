#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(eclipse_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]

extern crate alloc;

use alloc::{
    format,
    string::{String, ToString},
};
use core::{fmt::Write, panic::PanicInfo};
use spin::Mutex;
use uefi::{
    boot::{get_handle_for_protocol, open_protocol_exclusive},
    prelude::*,
    proto::console::text::Output,
};

use eclipse_os::{
    pc_speaker::{self, init_pc_speaker, test_pc_speaker, MusicNote}, serial::{info, serial_write_str}, time::{self, PerformanceCounter}, uefi_text_buffer::print_message, OutputForced, TEXT_OUTPUT
};

mod bump_allocator;
use bump_allocator::BumpAllocator;

// ================================================================================================
// CONSTANTS
// ================================================================================================

const HEAP_SIZE: usize = 4096;
const TIMER_TEST_ITERATIONS: u64 = 50_000_000;
const STATUS_UPDATE_INTERVAL: u64 = 50_000_000;
const PERFORMANCE_TEST_CYCLES: u32 = 1_000_000;
const STARTUP_DELAY_CYCLES: u32 = 1_000_000;
const RTC_TEST_DELAY_CYCLES: u32 = 10_000_000;

// ================================================================================================
// GLOBAL ALLOCATOR
// ================================================================================================

#[global_allocator]
static GLOBAL: BumpAllocator<HEAP_SIZE> = BumpAllocator::new();

// ================================================================================================
// ENTRY POINT
// ================================================================================================

#[entry]
fn efi_main() -> Status {
    if let Err(status) = initialize_uefi() {
        return status;
    }

    print_message("Eclipse OS Booting...\n");
    show_current_time();
    
    info("efi_main: Calling kernel_main\n");
    kernel_main()
}

// ================================================================================================
// INITIALIZATION FUNCTIONS
// ================================================================================================

fn initialize_uefi() -> Result<(), Status> {
    info("efi_main: Entered UEFI entry point\n");

    // Initialize UEFI helpers
    if let Err(e) = uefi::helpers::init() {
        info("efi_main: UEFI helpers init failed\n");
        return Err(e.status());
    }
    info("efi_main: UEFI helpers initialized\n");

    // Setup text output
    setup_text_output()?;
    
    info("efi_main: Using bump allocator for heap initialization\n");
    Ok(())
}

fn setup_text_output() -> Result<(), Status> {
    let handle = get_handle_for_protocol::<Output>()
        .map_err(|_| Status::NOT_FOUND)?;
    
    let mut output = open_protocol_exclusive::<Output>(handle)
        .map_err(|_| Status::DEVICE_ERROR)?;

    let raw_output = OutputForced(&mut *output as *mut Output);
    TEXT_OUTPUT.call_once(|| Mutex::new(raw_output));

    // Reset output if available
    if let Some(mutex) = TEXT_OUTPUT.get() {
        let output = mutex.lock();
        unsafe {
            if let Some(out) = output.0.as_mut() {
                out.reset(false).map_err(|_| Status::DEVICE_ERROR)?;
            } else {
                info("Failed to get mutable Output for reset!\n");
            }
        }
    }

    Ok(())
}

fn initialize_core_systems() {
    info("kernel_main: Calling eclipse_os::init()\n");
    eclipse_os::init();
    show_current_time();
}

fn initialize_time_system() {
    info("kernel_main: Initializing time system\n");
    print_message("Initializing time system...\n");
    
    time::init();
    info("kernel_main: Time system initialized successfully\n");
    print_message("Time System: [OK]\n");
    
    show_current_time();
}

// ================================================================================================
// KERNEL MAIN
// ================================================================================================

fn kernel_main() -> ! {
    info("kernel_main: Entered kernel_main\n");
    print_message("Eclipse OS - Initializing...\n");

    // Show initial time
    show_current_time();

    // Initialize core systems
    initialize_core_systems();

    // Initialize and test time system
    initialize_time_system();

    // Wait for timer to stabilize
    wait_for_timer_startup();

    // Display system information
    display_system_info();

    // Run system tests
    run_system_tests();

    print_message("System Ready!\n");
    
    test_pc_speaker();

    // Main system loop
    main_system_loop();
}

fn wait_for_timer_startup() {
    for _ in 0..STARTUP_DELAY_CYCLES {
        core::hint::spin_loop();
    }
}

fn display_system_info() {
    display_time_info();
    print_message("Hello from Eclipse OS!\n");
    display_time_info();
}

fn run_system_tests() {
    test_time_functions();
}

fn main_system_loop() -> ! {
    let mut counter = 0u64;
    
    loop {
        if counter % STATUS_UPDATE_INTERVAL == 0 {
            display_status_update(counter);
        }
        
        counter += 1;
        core::hint::spin_loop();
    }
}

// ================================================================================================
// STATUS AND DISPLAY FUNCTIONS
// ================================================================================================

fn display_status_update(counter: u64) {
    let loop_msg = format!("=== Loop iteration {} ===", counter);
    print_message(&loop_msg);
    
    show_current_time();
    display_timer_status();
    display_uptime_status();
    print_message(""); // Empty line
}

fn display_timer_status() {
    let current_ticks = time::get_ticks();
    let timer_msg = format!("Timer ticks: {}", current_ticks);
    print_message(&timer_msg);
}

fn display_uptime_status() {
    let uptime_seconds = time::get_uptime_seconds();
    if uptime_seconds > 0 {
        let uptime_str = time::get_uptime_string();
        let uptime_msg = format!("Uptime: {}", uptime_str);
        print_message(&uptime_msg);
    } else {
        print_message("Timer not running - no uptime available");
    }
}

fn show_current_time() {
    let utc_time = time::get_current_time();
    let local_time = time::get_current_time_local();
    let tz_offset = time::get_timezone_offset();
    
    let utc_str = format!("UTC Time: {}", utc_time);
    print_message(&utc_str);
    
    let local_str = format!(
        "Local Time: {} (UTC{}{})",
        local_time,
        if tz_offset >= 0 { "+" } else { "" },
        tz_offset
    );
    print_message(&local_str);
    
    info(&format!(
        "UTC: {}, Local: {}, TZ: {}\n", 
        utc_time, local_time, tz_offset
    ));
}

// ================================================================================================
// TIME INFORMATION DISPLAY
// ================================================================================================

fn display_time_info() {
    info("display_time_info: Displaying time information\n");
    print_message("=== TIME INFORMATION ===");
    
    show_current_time();
    display_boot_time();
    display_timer_info();
    display_cpu_frequency_info();
    
    print_message("========================");
    print_message(""); // Empty line
}

fn display_boot_time() {
    if let Some(boot_time_utc) = time::get_boot_time() {
        if let Some(boot_time_local) = time::get_boot_time_local() {
            let boot_str = format!("Boot Time UTC: {}", boot_time_utc);
            print_message(&boot_str);
            
            let boot_local_str = format!("Boot Time Local: {}", boot_time_local);
            print_message(&boot_local_str);
            
            info(&format!(
                "Boot time UTC: {}, Local: {}\n", 
                boot_time_utc, boot_time_local
            ));
        }
    } else {
        print_message("Boot Time: Not available");
    }
}

fn display_timer_info() {
    let ticks = time::get_ticks();
    let ticks_str = format!("Timer Ticks: {}", ticks);
    print_message(&ticks_str);
    info(&format!("Timer ticks: {}\n", ticks));

    if ticks > 0 {
        display_uptime_info();
        display_precision_info();
    } else {
        print_message("Timer not running - uptime calculations unavailable");
    }
}

fn display_uptime_info() {
    let uptime_seconds = time::get_uptime_seconds();
    let uptime_str = time::get_uptime_string();
    let uptime_display = format!("Uptime: {} ({}s)", uptime_str, uptime_seconds);
    print_message(&uptime_display);
    info(&format!("System uptime: {} seconds\n", uptime_seconds));
}

fn display_precision_info() {
    let time_ms = time::get_time_ms();
    let time_ns = time::get_time_ns();
    let precision_str = format!("Precision: {}ms / {}ns", time_ms, time_ns);
    print_message(&precision_str);
    info(&format!("Time precision: {}ms, {}ns\n", time_ms, time_ns));
}

fn display_cpu_frequency_info() {
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
}

// ================================================================================================
// TIME TESTING FUNCTIONS
// ================================================================================================

fn test_time_functions() {
    info("test_time_functions: Testing time functions\n");
    print_message("=== TESTING TIME FUNCTIONS ===");

    test_multiple_rtc_reads();
    test_timer_ticks();
    test_performance_counter();

    print_message("=== TIME FUNCTION TESTS COMPLETED ===");
    print_message("");
}

fn test_multiple_rtc_reads() {
    print_message("Test 1: Multiple RTC reads");
    
    for i in 1..=3 {
        let test_msg = format!("RTC Read #{}", i);
        print_message(&test_msg);
        show_current_time();
        
        // Manual delay using CPU cycles
        for _ in 0..RTC_TEST_DELAY_CYCLES {
            core::hint::spin_loop();
        }
    }
}

fn test_timer_ticks() {
    print_message("Test 2: Timer tick test");
    
    let start_ticks = time::get_ticks();
    print_message(&format!("Start ticks: {}", start_ticks));
    
    // Wait and check again
    for _ in 0..TIMER_TEST_ITERATIONS {
        core::hint::spin_loop();
    }
    
    let end_ticks = time::get_ticks();
    print_message(&format!("End ticks: {}", end_ticks));
    
    if end_ticks > start_ticks {
        print_message("Timer is working!");
    } else {
        print_message("Timer is NOT working - interrupts may not be enabled");
    }
}

fn test_performance_counter() {
    print_message("Test 3: Performance counter");
    
    let counter = PerformanceCounter::new();
    
    for _ in 0..PERFORMANCE_TEST_CYCLES {
        core::hint::spin_loop();
    }
    
    let elapsed_ns = counter.elapsed_ns();
    let elapsed_us = counter.elapsed_us();
    let elapsed_ms = counter.elapsed_ms();
    
    let perf_result = format!(
        "Performance: {}ns / {}us / {}ms", 
        elapsed_ns, elapsed_us, elapsed_ms
    );
    print_message(&perf_result);
    
    info(&format!(
        "Performance test: {}ns, {}us, {}ms\n", 
        elapsed_ns, elapsed_us, elapsed_ms
    ));
}

// ================================================================================================
// UTILITY FUNCTIONS
// ================================================================================================

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

fn manual_delay(ms: u32) {
    info(&format!("Manual delay: {}ms\n", ms));
    time::delay_ms(ms as u64);
}

#[allow(clippy::eq_op)]
fn trivial_assertion() -> Result<(), ()> {
    if 1 == 1 { 
        Ok(()) 
    } else { 
        Err(()) 
    }
}

// ================================================================================================
// PANIC HANDLERS
// ================================================================================================

#[cfg(not(test))]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    handle_panic(panic_info);
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    eclipse_os::test_panic_handler(info);
}

fn handle_panic(panic_info: &PanicInfo) -> ! {
    info("panic: Kernel panic occurred!\n");
    
    print_message("KERNEL PANIC: ");
    print_message(&format!("{}", panic_info.message()));
    
    if let Some(location) = panic_info.location() {
        print_message(&format!(
            " at {}:{}", 
            location.file(), 
            location.line()
        ));
        info(&format!(
            "panic: at {}:{}\n", 
            location.file(), 
            location.line()
        ));
    }
    
    print_message("\n");
    print_panic_info_serial(panic_info);
    
    loop {
        core::hint::spin_loop();
    }
}

pub fn print_panic_info_serial(panic_info: &PanicInfo) {
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

// ================================================================================================
// ASYNC FUNCTIONS (FOR FUTURE USE)
// ================================================================================================

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