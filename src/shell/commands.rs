use alloc::string::String;
use core::str::SplitWhitespace;
use crate::{vprintln, QemuExitCode, exit_qemu};
use crate::video_buffer::{self, Color, clear_screen};
use crate::time;
use crate::fs;
use crate::shutdown;
use crate::text_editor::{self, express_editor};
use crate::intereperter::run::run_example;

pub fn about() {
    video_buffer::set_color(Color::Cyan, Color::Black);
    vprintln!("\nEclipseOS");
    video_buffer::set_color(Color::White, Color::Black);
    vprintln!("A simple operating system written in Rust");
    vprintln!("Developed as a learning project");
    vprintln!("Type 'help' for available commands");
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
        vprintln!("[Filtered content]");
    } else {
        vprintln!("{}", trimmed_output);
    }
}

pub fn express() {
    clear();
    express_editor::init_editor();
}

pub fn hello() {
    vprintln!("Hello");
}

pub fn help() {
    vprintln!("Available commands:");
    vprintln!("  about    - Display information about EclipseS");
    vprintln!("  clear    - Clear the screen");
    vprintln!("  ls       - Lists the contents of a directory");
    vprintln!("  echo     - Display a line of text");
    vprintln!("  hello    - Displays \"Hello\"");
    vprintln!("  help     - Display this help message");
    vprintln!("  shutdown - Shutsdown the computer");
    vprintln!("  time     - Displays current time");
    vprintln!("  version  - Display the current version of EclipseOS");
    vprintln!("  express  - Activates the express text editor");
    vprintln!("  test     - Test the express text editor");
}

pub fn test() {
    run_example();
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

    vprintln!("current date: {}!",date_str);
    vprintln!("current time: {}!",time_str);
}

pub fn version() {
    vprintln!("EclipseOS v0.1.0");
}

pub fn halt() {
    vprintln!("System halted.");
    loop {
        x86_64::instructions::hlt();
    }
}