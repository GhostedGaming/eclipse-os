use core::arch::asm;
use crate::cpu::cpuid;
use crate::rtc::{self, DateTime};

// PIT constants
const PIT_FREQUENCY: u32 = 1193182; // PIT base frequency in Hz
const DESIRED_FREQUENCY: u32 = 1000; // 1000 Hz = 1ms per tick

// Timezone configuration - US East Coast
const LOCAL_TIMEZONE_OFFSET_HOURS: i8 = -5; // EST is UTC-5 (or -4 for EDT during daylight saving)

static mut TICKS: u64 = 0;
static mut CPU_FREQUENCY_HZ: Option<u64> = None;
static mut NANOSECONDS_PER_TICK: u64 = 1_000_000; // 1ms in nanoseconds
static mut BOOT_TIME: Option<DateTime> = None;

pub fn init() {
    // Initialize CPU info first
    cpuid::init_cpu_info();

    // Get CPU frequency for timing calculations
    if let Some(cpu_info) = cpuid::get_cpu_info()
        && let Some(freq) = cpu_info.get_timing_frequency_hz()
    {
        unsafe {
            CPU_FREQUENCY_HZ = Some(freq);
        }
        crate::println!("CPU frequency: {} Hz", freq);
    }

    // Read initial time from RTC
    let current_time = rtc::get_current_time();
    unsafe {
        BOOT_TIME = Some(current_time);
    }
    crate::println!("System time: {}", current_time);

    // Configure PIT for precise timing
    configure_pit_timer();
}

/// Write to PIT command port (0x43)
fn pit_command_port(value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") 0x43u16,
            in("al") value,
            options(nomem, nostack)
        );
    }
}

/// Write to PIT data port channel 0 (0x40)
fn pit_data_port_ch0(value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") 0x40u16,
            in("al") value,
            options(nomem, nostack)
        );
    }
}

fn configure_pit_timer() {
    // Calculate the divisor for the desired frequency
    let divisor = PIT_FREQUENCY / DESIRED_FREQUENCY;
    
    crate::println!(
        "Configuring PIT: base_freq={} Hz, desired_freq={} Hz, divisor={}",
        PIT_FREQUENCY,
        DESIRED_FREQUENCY,
        divisor
    );

    unsafe {
        // Command port: Channel 0, Access mode lobyte/hibyte, Mode 2 (rate generator), Binary mode
        pit_command_port(0x34u8);

        // Send the divisor (low byte first, then high byte)
        pit_data_port_ch0((divisor & 0xFF) as u8);
        pit_data_port_ch0((divisor >> 8) as u8);
    }

    unsafe {
        NANOSECONDS_PER_TICK = 1_000_000_000 / DESIRED_FREQUENCY as u64;
    }

    crate::println!(
        "PIT configured for {} Hz ({} ns per tick)",
        DESIRED_FREQUENCY,
        unsafe { NANOSECONDS_PER_TICK }
    );
}

pub fn tick() {
    unsafe {
        TICKS += 1;
    }
}

pub fn get_ticks() -> u64 {
    unsafe { TICKS }
}

pub fn get_time_ms() -> u64 {
    unsafe { TICKS * NANOSECONDS_PER_TICK / 1_000_000 }
}

pub fn get_time_ns() -> u64 {
    unsafe { TICKS * NANOSECONDS_PER_TICK }
}

// High precision timing using TSC if available
pub fn get_precise_time_ns() -> Option<u64> {
    if let Some(cpu_freq) = unsafe { CPU_FREQUENCY_HZ } {
        let tsc = unsafe { core::arch::x86_64::_rdtsc() };
        Some((tsc * 1_000_000_000) / cpu_freq)
    } else {
        None
    }
}

pub fn get_current_time() -> DateTime {
    rtc::get_current_time()
}

/// Get current time adjusted for local timezone
pub fn get_current_time_local() -> DateTime {
    let utc_time = rtc::get_current_time();
    adjust_time_for_timezone(utc_time)
}

/// Get the timezone offset in hours
pub fn get_timezone_offset() -> i8 {
    LOCAL_TIMEZONE_OFFSET_HOURS
}

/// Simple timezone adjustment (basic implementation)
fn adjust_time_for_timezone(mut datetime: DateTime) -> DateTime {
    let mut hour = datetime.hour as i8 + LOCAL_TIMEZONE_OFFSET_HOURS;
    let mut day = datetime.day;
    let mut month = datetime.month;
    let mut year = datetime.year;
    
    // Handle day rollover
    if hour < 0 {
        hour += 24;
        day -= 1;
        if day == 0 {
            // Go to previous month
            month -= 1;
            if month == 0 {
                month = 12;
                year -= 1;
            }
            // Set to last day of previous month (simplified)
            day = days_in_month(month, year);
        }
    } else if hour >= 24 {
        hour -= 24;
        day += 1;
        let days_this_month = days_in_month(month, year);
        if day > days_this_month {
            day = 1;
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }
    }
    
    datetime.hour = hour as u8;
    datetime.day = day;
    datetime.month = month;
    datetime.year = year;
    datetime
}

/// Get number of days in a month (simplified, doesn't handle all leap year edge cases)
fn days_in_month(month: u8, year: u16) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30, // fallback
    }
}

/// Simple leap year check
fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn get_uptime_seconds() -> u64 {
    get_time_ms() / 1000
}

pub fn delay_ms(ms: u64) {
    let start_ticks = get_ticks();
    let target_ticks = start_ticks + ms;
    while get_ticks() < target_ticks {
        core::hint::spin_loop();
    }
}

pub fn delay_us(microseconds: u64) {
    let start_time = get_time_ns();
    let target_time = start_time + (microseconds * 1000); // Convert to nanoseconds
    while get_time_ns() < target_time {
        core::hint::spin_loop();
    }
}

pub fn delay(milliseconds: f64) {
    if milliseconds > 0.0 {
        delay_ms(milliseconds as u64);
    }
}

pub fn precise_delay_ns(nanoseconds: f64) {
    if let Some(cpu_freq) = get_cpu_frequency_hz() {
        let start_tsc = unsafe { core::arch::x86_64::_rdtsc() };
        let cycles_to_wait = (nanoseconds * cpu_freq as f64) / 1_000_000_000.0;
        let target_tsc = start_tsc + cycles_to_wait as u64;
        
        while unsafe { core::arch::x86_64::_rdtsc() } < target_tsc {
            core::hint::spin_loop();
        }
    } else {
        // Fallback to regular delay
        delay_ms((nanoseconds / 1_000_000.0) as u64);
    }
}

pub fn precise_delay_us(microseconds: u64) {
    precise_delay_ns((microseconds * 1000) as f64);
}

// Time sync task for your async executor
pub async fn time_sync_task() {
    loop {
        // Just yield to other tasks, don't print anything automatically
        crate::task::yield_now().await;
    }
}

pub fn get_cpu_frequency_hz() -> Option<u64> {
    unsafe { CPU_FREQUENCY_HZ }
}

/// Read from CMOS/RTC register
pub fn read_cmos(register: u8) -> u8 {
    unsafe {
        // Write register number to 0x70
        asm!(
            "out dx, al",
            in("dx") 0x70u16,
            in("al") register,
            options(nomem, nostack)
        );
        
        // Read data from 0x71
        let value: u8;
        asm!(
            "in al, dx",
            in("dx") 0x71u16,
            out("al") value,
            options(nomem, nostack)
        );
        value
    }
}

/// Write to CMOS/RTC register
pub fn write_cmos(register: u8, value: u8) {
    unsafe {
        // Write register number to 0x70
        asm!(
            "out dx, al",
            in("dx") 0x70u16,
            in("al") register,
            options(nomem, nostack)
        );
        
        // Write data to 0x71
        asm!(
            "out dx, al",
            in("dx") 0x71u16,
            in("al") value,
            options(nomem, nostack)
        );
    }
}

/// Get system uptime as a formatted string
pub fn get_uptime_string() -> alloc::string::String {
    let uptime_seconds = get_uptime_seconds();
    let hours = uptime_seconds / 3600;
    let minutes = (uptime_seconds % 3600) / 60;
    let seconds = uptime_seconds % 60;
    
    alloc::format!("{}:{:02}:{:02}", hours, minutes, seconds)
}

/// Get boot time
pub fn get_boot_time() -> Option<DateTime> {
    unsafe { BOOT_TIME }
}

/// Get boot time in local timezone
pub fn get_boot_time_local() -> Option<DateTime> {
    unsafe { BOOT_TIME.map(adjust_time_for_timezone) }
}

/// Performance counter for benchmarking
pub struct PerformanceCounter {
    start_ticks: u64,
    start_tsc: Option<u64>,
}

impl PerformanceCounter {
    pub fn new() -> Self {
        let start_tsc = if unsafe { CPU_FREQUENCY_HZ }.is_some() {
            Some(unsafe { core::arch::x86_64::_rdtsc() })
        } else {
            None
        };
        
        Self {
            start_ticks: get_ticks(),
            start_tsc,
        }
    }
    
    pub fn elapsed_ms(&self) -> u64 {
        let current_ticks = get_ticks();
        (current_ticks - self.start_ticks) * unsafe { NANOSECONDS_PER_TICK } / 1_000_000
    }
    
    pub fn elapsed_us(&self) -> u64 {
        let current_ticks = get_ticks();
        (current_ticks - self.start_ticks) * unsafe { NANOSECONDS_PER_TICK } / 1_000
    }
    
    pub fn elapsed_ns(&self) -> u64 {
        if let (Some(start_tsc), Some(cpu_freq)) = (self.start_tsc, unsafe { CPU_FREQUENCY_HZ }) {
            let current_tsc = unsafe { core::arch::x86_64::_rdtsc() };
            ((current_tsc - start_tsc) * 1_000_000_000) / cpu_freq
        } else {
            let current_ticks = get_ticks();
            (current_ticks - self.start_ticks) * unsafe { NANOSECONDS_PER_TICK }
        }
    }
}

impl Default for PerformanceCounter {
    fn default() -> Self {
        Self::new()
    }
}