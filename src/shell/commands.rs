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
}

pub fn ls() {
    println!("Reading disk contents...");
    
    // Create a filesystem instance
    let fs = fs::fat32::FileSystem { 
        root_cluster: 2,
        cluster_size: 512,
        data_area_offset: 32768,
    };
    
    // Define a read_sector function with more careful ATA access
    fn read_sector(sector: u32, buffer: &mut [u8]) {
        use x86_64::instructions::port::Port;
        use core::sync::atomic::{fence, Ordering};
        
        // Ensure buffer is at least 512 bytes
        if buffer.len() < 512 {
            return;
        }
        
        unsafe {
            // ATA ports
            let mut data_port = Port::<u16>::new(0x1F0);
            let mut error_port = Port::<u8>::new(0x1F1);
            let mut sector_count_port = Port::<u8>::new(0x1F2);
            let mut lba_low_port = Port::<u8>::new(0x1F3);
            let mut lba_mid_port = Port::<u8>::new(0x1F4);
            let mut lba_high_port = Port::<u8>::new(0x1F5);
            let mut device_port = Port::<u8>::new(0x1F6);
            let mut command_port = Port::<u8>::new(0x1F7);
            let mut control_port = Port::<u8>::new(0x3F6);
            
            // Disable interrupts
            control_port.write(0x02);
            
            // Wait until drive is ready
            loop {
                let status = command_port.read();
                if (status & 0x80) == 0 && (status & 0x40) != 0 {
                    break;
                }
            }
            
            // Set up for reading
            sector_count_port.write(1);
            lba_low_port.write((sector & 0xFF) as u8);
            lba_mid_port.write(((sector >> 8) & 0xFF) as u8);
            lba_high_port.write(((sector >> 16) & 0xFF) as u8);
            
            // LBA mode, primary drive, high 4 bits of LBA
            device_port.write(0xE0 | (((sector >> 24) & 0x0F) as u8));
            
            // Send read command
            command_port.write(0x20);
            
            // Memory barrier
            fence(Ordering::SeqCst);
            
            // Wait for data
            loop {
                let status = command_port.read();
                if (status & 0x80) == 0 && (status & 0x08) != 0 {
                    break;
                }
                if (status & 0x01) != 0 {
                    // Error occurred
                    let _error = error_port.read();
                    return;
                }
            }
            
            // Read data
            for i in 0..256 {  // 512 bytes = 256 words
                let data = data_port.read();
                buffer[i*2] = data as u8;
                buffer[i*2 + 1] = (data >> 8) as u8;
            }
            
            // Memory barrier
            fence(Ordering::SeqCst);
        }
    }
    
    // Call list_directory with both required arguments
    let entries = fs.list_directory(fs.root_cluster, read_sector);
    
    if entries.is_empty() {
        println!("Directory is empty");
    } else {
        println!("Directory contents:");
        for entry in entries {
            println!("  {}", entry);
        }
    }
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