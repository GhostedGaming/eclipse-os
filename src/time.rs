use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::instructions::port::Port;
use alloc::string::String;
use core::fmt::Write;
use crate::task::{Task, executor::Executor};

// Store system time as milliseconds since epoch (Unix timestamp)
static SYSTEM_TIME: AtomicU64 = AtomicU64::new(0);
// Store ticks since boot
static TICKS_SINCE_BOOT: AtomicU64 = AtomicU64::new(0);
// tick per second (1000 ticks per second for millisecond precision)
static TICKS_PER_SECOND: u64 = 1000;

// CMOS RTC registers
const CMOS_ADDRESS: u16 = 0x70;
const CMOS_DATA: u16 = 0x71;

// CMOS register addresses
const CMOS_SECONDS: u8 = 0x00;
const CMOS_MINUTES: u8 = 0x02;
const CMOS_HOURS: u8 = 0x04;
const CMOS_DAY: u8 = 0x07;
const CMOS_MONTH: u8 = 0x08;
const CMOS_YEAR: u8 = 0x09;
const CMOS_CENTURY: u8 = 0x32; // May be different on some systems
const CMOS_STATUS_A: u8 = 0x0A;
const CMOS_STATUS_B: u8 = 0x0B;

/// Initialize the time subsystem
pub fn init() {
    // Read initial time from RTC
    let time = read_rtc_time();
    
    if time < 1743471294000 { // Roughly year 2020 in milliseconds
        // Use a hardcoded recent timestamp as fallback
        SYSTEM_TIME.store(1672531200000, Ordering::SeqCst); // Jan 1, 2023 in milliseconds
    } else {
        SYSTEM_TIME.store(time, Ordering::SeqCst);
    }
}

/// Increment the system tick counter (called by timer interrupt handler)
pub fn tick() {
    let ticks = TICKS_SINCE_BOOT.fetch_add(1, Ordering::SeqCst) + 1;

    if ticks % TICKS_PER_SECOND == 0 {
        // Normal second increment (1000 milliseconds)
        let current = SYSTEM_TIME.load(Ordering::SeqCst);
        SYSTEM_TIME.store(current + 1000, Ordering::SeqCst);
    } else {
        // Increment milliseconds
        let current = SYSTEM_TIME.load(Ordering::SeqCst);
        SYSTEM_TIME.store(current + 1, Ordering::SeqCst);
    }
}

/// Get current time as milliseconds since epoch
pub fn current_time() -> u64 {
    SYSTEM_TIME.load(Ordering::SeqCst)
}

/// Get ticks since boot
pub fn ticks() -> u64 {
    TICKS_SINCE_BOOT.load(Ordering::SeqCst)
}

/// Async task to periodically sync with RTC
pub async fn time_sync_task() {
    loop {
        // Sync with RTC every minute
        let rtc_time = read_rtc_time();
        if rtc_time > 1600000000000 {
            SYSTEM_TIME.store(rtc_time, Ordering::SeqCst);
        }
        
        // Sleep for 60 seconds before next sync
        // This is a simple async sleep implementation
        let current_ticks = ticks();
        let target_ticks = current_ticks + 60000; // 60 seconds at 1000Hz
        
        while ticks() < target_ticks {
            // Yield to other tasks
            crate::task::yield_now().await;
        }
    }
}

/// Read time from RTC (Real-Time Clock)
fn read_rtc_time() -> u64 {
    // Wait until RTC is not updating
    wait_for_rtc();
    
    // Read RTC values
    let seconds = read_cmos_register(CMOS_SECONDS);
    let minutes = read_cmos_register(CMOS_MINUTES);
    let hours = read_cmos_register(CMOS_HOURS);
    let day = read_cmos_register(CMOS_DAY);
    let month = read_cmos_register(CMOS_MONTH);
    let year = read_cmos_register(CMOS_YEAR);
    let century = read_cmos_register(CMOS_CENTURY);
    
    // Read status register B to check if values are BCD or binary
    let status_b = read_cmos_register(CMOS_STATUS_B);
    let is_bcd = (status_b & 0x04) == 0;
    
    // Convert from BCD to binary if needed
    let seconds = if is_bcd { bcd_to_binary(seconds) } else { seconds };
    let minutes = if is_bcd { bcd_to_binary(minutes) } else { minutes };
    let hours = if is_bcd { bcd_to_binary(hours) } else { hours };
    let day = if is_bcd { bcd_to_binary(day) } else { day };
    let month = if is_bcd { bcd_to_binary(month) } else { month };
    let year = if is_bcd { bcd_to_binary(year) } else { year };
    let century = if is_bcd { bcd_to_binary(century) } else { century };
    
    // Calculate full year
    let year = (century as u16 * 100 + year as u16) as u64;
    
    // Convert to Unix timestamp in milliseconds (simplified algorithm)
    // This is a basic implementation and doesn't account for leap seconds
    let mut timestamp: u64 = 0;
    
    // Add milliseconds from years
    for y in 1970..year {
        timestamp += 31536000000; // 365 days in milliseconds
        if is_leap_year(y as u16) {
            timestamp += 86400000; // Add leap day in milliseconds
        }
    }
    
    // Add milliseconds from months
    let days_in_month = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month {
        timestamp += days_in_month[m as usize] as u64 * 86400000;
        if m == 2 && is_leap_year(year as u16) {
            timestamp += 86400000; // Add leap day in February in milliseconds
        }
    }
    
    // Add milliseconds from days
    timestamp += (day as u64 - 1) * 86400000;
    
    // Add milliseconds from hours, minutes, and seconds
    timestamp += hours as u64 * 3600000;
    timestamp += minutes as u64 * 60000;
    timestamp += seconds as u64 * 1000;
    
    timestamp
}

/// Wait until RTC is not updating
fn wait_for_rtc() {
    let mut address_port = Port::<u8>::new(CMOS_ADDRESS);
    let mut data_port = Port::<u8>::new(CMOS_DATA);
    
    unsafe {
        // Select status register A
        address_port.write(CMOS_STATUS_A);
        
        // Wait until update-in-progress flag is clear
        while (data_port.read() & 0x80) != 0 {
            // Keep checking
        }
    }
}

/// Read a value from CMOS/RTC register
fn read_cmos_register(register: u8) -> u8 {
    let mut address_port = Port::new(CMOS_ADDRESS);
    let mut data_port = Port::new(CMOS_DATA);
    
    unsafe {
        // Select the register
        address_port.write(register);
        
        // Read the value
        data_port.read()
    }
}

/// Convert BCD to binary
fn bcd_to_binary(bcd: u8) -> u8 {
    (bcd & 0x0F) + ((bcd >> 4) * 10)
}

/// Check if a year is a leap year
fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Format time as a string (HH:MM:SS.sss)
pub fn format_time() -> String {
    let timestamp = current_time();
    
    // Extract hours, minutes, seconds, milliseconds
    let milliseconds = timestamp % 1000;
    let seconds = (timestamp / 1000) % 60;
    let minutes = (timestamp / 60000) % 60;
    let hours = (timestamp / 3600000) % 24;
    
    let mut time_str = String::new();
    write!(time_str, "{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds).unwrap();
    
    time_str
}

/// Format date as a string (YYYY-MM-DD)
pub fn format_date() -> String {
    let timestamp = current_time();
    
    // This is a simplified algorithm to extract date components
    
    // Start from 1970-01-01
    let mut year = 1970;
    let mut month = 1;
    let mut day = 1;
    
    // Calculate days since epoch
    let mut days_since_epoch = timestamp / 86400000;
    
    // Calculate year
    while days_since_epoch >= 365 {
        if is_leap_year(year) && days_since_epoch >= 366 {
            days_since_epoch -= 366;
            year += 1;
        } else if !is_leap_year(year) {
            days_since_epoch -= 365;
            year += 1;
        } else {
            break;
        }
    }
    
    // Calculate month and day
    let days_in_month = [
        0, 31, 
        if is_leap_year(year) { 29 } else { 28 }, 
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31
    ];
    
    while days_since_epoch >= days_in_month[month] {
        days_since_epoch -= days_in_month[month];
        month += 1;
    }
    
    day += days_since_epoch as u8;
    
    let mut date_str = String::new();
    write!(date_str, "{:04}-{:02}-{:02}", year, month, day).unwrap();
    
    date_str
}
