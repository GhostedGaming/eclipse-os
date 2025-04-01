use alloc::string::String;
use core::str::SplitWhitespace;
use crate::{print, println, QemuExitCode, exit_qemu};
use crate::vga_buffer::{self, Color, clear_screen};
use crate::time;
use crate::shutdown;


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
    
    while let Some(arg) = args.next() {
        output.push_str(arg);
        output.push(' ');
    }
    
    println!("{}", output.trim_end());
}

pub fn hello() {
    println!("Hello");
}

pub fn help() {
    println!("Available commands:");
    println!("  about    - Display information about EclipseOS");
    println!("  clear    - Clear the screen");
    println!("  disk(wip)     - Displays a list of different disk drives");
    println!("  echo     - Display a line of text");
    println!("  hello    - Displays \"Hello\"");
    println!("  help     - Display this help message");
    println!("  shutdown - Shutsdown the computer");
    println!("  time(wip)     - Displays current time");
    println!("  version  - Display the current version of EclipseOS");
}

pub fn shutdown() {
    shutdown::shutdown();
}
pub fn qemu_shutdown() {
    exit_qemu(QemuExitCode::Success);
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