use alloc::string::String;
use core::str::SplitWhitespace;
use crate::cpu::cpuid;
use crate::{exit_qemu, println, vga_buffer, QemuExitCode};
use crate::time;
use crate::shutdown;
use crate::text_editor::express_editor;
use crate::intereperter::run::run_example;

pub fn about() {
    vga_buffer::set_color(vga_buffer::Color::Cyan, vga_buffer::Color::Black);
    println!("\nEclipseOS");
    vga_buffer::set_color(vga_buffer::Color::White, vga_buffer::Color::Black);
    println!("A simple operating system written in Rust");
    println!("Developed as a learning project");
    println!("Type 'help' for available commands");
}

pub fn clear() {
    vga_buffer::clear_screen();
}

pub fn echo(mut args: SplitWhitespace) {
    // Use a fixed-size buffer instead of dynamic String allocation
    let mut output_bytes = [0u8; 256]; // Fixed 256-byte buffer
    let mut len = 0;
    
    let slurs = [
        "fgewfew", "hgergre",
    ];
    
    // Build output in fixed buffer
    while let Some(arg) = args.next() {
        for byte in arg.bytes() {
            if len < output_bytes.len() - 1 {
                output_bytes[len] = byte;
                len += 1;
            }
        }
        if len < output_bytes.len() - 1 {
            output_bytes[len] = b' ';
            len += 1;
        }
    }
    
    // Convert to string slice for processing
    if let Ok(output_str) = core::str::from_utf8(&output_bytes[..len.saturating_sub(1)]) {
        let contains_slur = slurs.iter().any(|&slur| 
            output_str.to_lowercase().contains(slur)
        );
        
        if contains_slur {
            println!("[Filtered content]");
        } else {
            println!("{}", output_str);
        }
    } else {
        println!("Invalid UTF-8 in input");
    }
}

pub fn express() {
    clear();
    express_editor::init_editor();
}

pub fn hello() {
    println!("Hello");
}

pub fn help() {
    println!("Available commands:");
    println!("  about    - Display information about EclipseS");
    println!("  clear    - Clear the screen");
    println!("  ls       - Lists the contents of a directory");
    println!("  echo     - Display a line of text");
    println!("  hello    - Displays \"Hello\"");
    println!("  help     - Display this help message");
    println!("  shutdown - Shutsdown the computer");
    println!("  time     - Displays current time");
    println!("  version  - Display the current version of EclipseOS");
    println!("  express  - Activates the express text editor");
    println!("  test     - Test the express text editor");
}

pub fn test() {
    run_example();
}

pub fn cpuid() {
    cpuid::print_cpu_info();
}

pub fn qemu_shutdown() {
    exit_qemu(QemuExitCode::Success);
}

pub fn shutdown() {
    shutdown::shutdown();
}

pub fn time() {
    let current_time = crate::time::get_current_time();
    let uptime = crate::time::get_uptime_seconds();
    
    println!("Current time: {}", current_time);
    println!("Uptime: {}s", uptime);
}

pub fn date() {
    let current_time = crate::time::get_current_time();
    println!("{:04}-{:02}-{:02} {}", 
            current_time.year, 
            current_time.month, 
            current_time.day,
            current_time);
}

pub fn clock() {
    let current_time = crate::time::get_current_time();
    let (hour, minute, is_pm) = current_time.format_12h();
    let ampm = if is_pm { "PM" } else { "AM" };
    
    println!("╔════════════╗");
    println!("║  {:02}:{:02} {}  ║", hour, minute, ampm);
    println!("╚════════════╝");
}


pub fn version() {
    println!("EclipseOS v0.1.0");
}

pub fn halt() {
    println!("System halted.");
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn time_test() {
    use crate::time;
    
    println!("=== Timing Test ===");
    println!("Current ticks: {}", time::get_ticks());
    println!("Time (ms): {}", time::get_time_ms());
    println!("Time (ns): {}", time::get_time_ns());
    
    if let Some(precise_ns) = time::get_precise_time_ns() {
        println!("Precise time (ns): {}", precise_ns);
    }
    
    if let Some(cpu_freq) = time::get_cpu_frequency_hz() {
        println!("CPU frequency: {} Hz", cpu_freq);
    }
}
