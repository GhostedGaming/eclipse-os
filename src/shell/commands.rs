use alloc::string::String;
use core::str::SplitWhitespace;
use crate::{println, QemuExitCode, exit_qemu};
use crate::vga_buffer::{self, Color, clear_screen};
use crate::time;
use crate::shutdown;
use crate::fs::list_disks::print_disks;

pub fn about() {
    vga_buffer::set_color(Color::Cyan, Color::Black);
    println!("\nEclipseOS");
    vga_buffer::set_color(Color::White, Color::Black);
    println!("A simple operating system written in Rust");
    println!("Developed as a learning project");
    println!("Type 'help' for available commands");
}

pub fn clear() {
    clear_screen();
}

pub fn echo(mut args: SplitWhitespace) {
    let mut output = String::new();
    let slurs = [
        "fuck", "shit", "ass", "bitch", "damn", "crap", "hell", 
        "dick", "pussy", "nigger", "nigga", "cunt", "asshole", 
        "motherfucker", "bullshit", "bastard", "piss"
    ];
    
    while let Some(arg) = args.next() {
        output.push_str(arg);
        output.push(' ');
    }
    
    let trimmed_output = output.trim_end();
    
    // Check if the output contains any slurs
    let contains_slur = slurs.iter().any(|&slur| trimmed_output.to_lowercase().contains(slur));
    
    if contains_slur {
        println!("[Filtered content]");
    } else {
        println!("{}", trimmed_output);
    }
}

pub fn hello() {
    println!("Hello");
}

pub fn help() {
    println!("Available commands:");
    println!("  about    - Display information about EclipseOS");
    println!("  clear    - Clear the screen");
    println!("  disks    - Displays a list of different disk drives");
    println!("  echo     - Display a line of text");
    println!("  hello    - Displays \"Hello\"");
    println!("  help     - Display this help message");
    println!("  shutdown - Shutsdown the computer");
    println!("  time     - Displays current time");
    println!("  version  - Display the current version of EclipseOS");
}

pub fn disks() {
    print_disks();
}

pub fn qemu_shutdown() {
    exit_qemu(QemuExitCode::Success);
}

pub fn shutdown() {
    shutdown::shutdown();
}

pub fn time() {
    let time_str = time::format_time();
    let date_str = time::format_date();

    println!("current date: {}!",date_str);
    println!("current time: {}!",time_str);
}

pub fn version() {
    println!("EclipseOS v0.1.0");
}