use crate::cpu::cpuid;
use crate::rtc::{self, DateTime};
use x86_64::instructions::port::Port;

// PIT constants
const PIT_FREQUENCY: u32 = 1193182; // PIT base frequency in Hz
const DESIRED_FREQUENCY: u32 = 1000; // 1000 Hz = 1ms per tick

static mut TICKS: u64 = 0;
static mut CPU_FREQUENCY_HZ: Option<u64> = None;
static mut NANOSECONDS_PER_TICK: u64 = 1_000_000; // 1ms in nanoseconds
static mut BOOT_TIME: Option<DateTime> = None;

pub fn init() {
    // Initialize CPU info first
    cpuid::init_cpu_info();
    
    // Get CPU frequency for timing calculations
    if let Some(cpu_info) = cpuid::get_cpu_info() {
        if let Some(freq) = cpu_info.get_timing_frequency_hz() {
            unsafe {
                CPU_FREQUENCY_HZ = Some(freq);
            }
            crate::println!("CPU frequency: {} Hz", freq);
        }
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

fn configure_pit_timer() {
    // Calculate the divisor for the desired frequency
    let divisor = PIT_FREQUENCY / DESIRED_FREQUENCY;
    
    crate::println!("Configuring PIT: base_freq={} Hz, desired_freq={} Hz, divisor={}",
                    PIT_FREQUENCY, DESIRED_FREQUENCY, divisor);

    unsafe {
        // Command port: Channel 0, Access mode lobyte/hibyte, Mode 2 (rate generator), Binary mode
        let mut command_port = Port::new(0x43);
        command_port.write(0x34u8);

        // Data port for channel 0
        let mut data_port = Port::new(0x40);
        
        // Send the divisor (low byte first, then high byte)
        data_port.write((divisor & 0xFF) as u8);
        data_port.write((divisor >> 8) as u8);
    }

    unsafe {
        NANOSECONDS_PER_TICK = 1_000_000_000 / DESIRED_FREQUENCY as u64;
    }

    crate::println!("PIT configured for {} Hz ({} ns per tick)", DESIRED_FREQUENCY, unsafe { NANOSECONDS_PER_TICK });
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
// pub fn get_precise_time_ns() -> Option<u64> {
//     if let Some(cpu_freq) = unsafe { CPU_FREQUENCY_HZ } {
//         let tsc = unsafe { core::arch::x86_64::_rdtsc() };
//         Some((tsc * 1_000_000_000) / cpu_freq)
//     } else {
//         None
//     }
// }

pub fn get_current_time() -> DateTime {
    rtc::get_current_time()
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