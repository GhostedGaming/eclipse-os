pub mod commands;

use alloc::string::String;
use crate::{print, println, vga_buffer, QemuExitCode, exit_qemu};

pub struct Shell {
    input_buffer: String,
    cursor_position: usize,
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            input_buffer: String::new(),
            cursor_position: 0,
        }
    }

    pub fn process_keypress(&mut self, c: char) {
        match c {
            '\n' => {
                println!();
                self.execute_command();
                print!("eclipse> ");
                self.input_buffer.clear();
                self.cursor_position = 0;
            }
            '\u{8}' => { // Backspace
                if self.cursor_position > 0 {
                    self.input_buffer.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                    vga_buffer::backspace();
                }
            }
            c if c.is_ascii() && !c.is_control() => {
                self.input_buffer.insert(self.cursor_position, c);
                self.cursor_position += 1;
                print!("{}", c);
            }
            _ => {}
        }
    }

    fn execute_command(&self) {
        let input = self.input_buffer.trim();
        if input.is_empty() {
            return;
        }

        let mut parts = input.split_whitespace();
        let command = parts.next().unwrap_or("");
        
        match command {
            "help" => commands::help(),
            "echo" => commands::echo(parts),
            "clear" => commands::clear(),
            "about" => commands::about(),
            "version" => commands::version(),
            "hello" => commands::hello(),
            "shutdown" => commands::shutdown(),
            "qemu_shutdown" => exit_qemu(QemuExitCode::Success),
            "time" => commands::time(),
            "test" => commands::test(),
            "express" => commands::express(),
            "halt" => commands::halt(),
            "cpuid" => commands::cpuid(),
            _ => println!("Unknown command: {}. Type 'help' for available commands.", command),
        }
    }

    pub fn start(&mut self) {
        println!("EclipseOS Shell v0.1.0");
        println!("Type 'help' for available commands.");
        print!("eclipse> ");
    }
}