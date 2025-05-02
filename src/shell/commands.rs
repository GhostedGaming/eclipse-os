use alloc::string::String;
use core::str::SplitWhitespace;
use crate::{println, QemuExitCode, exit_qemu};
use crate::vga_buffer::{self, Color, clear_screen};
use crate::time;
use crate::fs;
use crate::shutdown;
use crate::text_editor::{self, express_editor};

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
        //disregard this its stupid
        "fgewfew", "hgergre",
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
    express_editor::test();
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