use crate::pc_speaker::{Melody, play_melody};
use crate::uefi_text_buffer::{backspace, print_char, print_message};
use alloc::format;
use alloc::string::String;

pub struct Shell {
    input_buffer: String,
    cursor_position: usize,
    prompt: &'static str,
    input_start_col: usize,
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            input_buffer: String::new(),
            cursor_position: 0,
            prompt: "eclipse-os> ",
            input_start_col: 0,
        }
    }

    pub fn start(&mut self) {
        self.show_prompt();
    }

    fn show_prompt(&mut self) {
        // Print prompt
        for ch in self.prompt.chars() {
            print_char(ch);
        }
        // Track where input starts (after the prompt)
        self.input_start_col = self.prompt.len();
    }

    pub fn process_keypress(&mut self, key: char) {
        match key {
            '\n' => {
                print_char('\n'); // Move to next line
                self.execute_command();
                self.input_buffer.clear();
                self.cursor_position = 0;
                self.show_prompt();
            }
            '\u{8}' => {
                // Backspace
                self.handle_backspace();
            }
            c if c.is_ascii() && !c.is_control() => {
                self.insert_char(c);
            }
            _ => {} // Ignore other characters
        }
    }

    fn insert_char(&mut self, c: char) {
        // Insert character at cursor position
        if self.cursor_position <= self.input_buffer.len() {
            self.input_buffer.insert(self.cursor_position, c);
            self.cursor_position += 1;

            // For simple insertion at the end, just print the character
            if self.cursor_position == self.input_buffer.len() {
                print_char(c);
            } else {
                // Need to redraw if inserting in the middle
                self.redraw_input_line();
            }
        } else {
            // Fallback: just append and print
            self.input_buffer.push(c);
            self.cursor_position = self.input_buffer.len();
            print_char(c);
        }
    }

    fn handle_backspace(&mut self) {
        // Only allow backspace if we have input to remove
        if self.cursor_position > 0 && !self.input_buffer.is_empty() {
            // Remove character before cursor
            self.cursor_position -= 1;
            self.input_buffer.remove(self.cursor_position);

            // Use the UEFI backspace function
            backspace();
        } else {
            // At beginning of input, just make a beep sound
            crate::pc_speaker::beep(500, 10);
        }
    }

    fn redraw_input_line(&mut self) {
        // This is simplified since UEFI text mode is more limited
        // We'll just reprint the entire line for now

        // Clear current line by printing spaces (simplified approach)
        for _ in 0..80 {
            print_char(' ');
        }

        // Print prompt again
        for ch in self.prompt.chars() {
            print_char(ch);
        }

        // Print the current input buffer
        for ch in self.input_buffer.chars() {
            print_char(ch);
        }
    }

    // Simplified cursor movement for UEFI mode
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            // UEFI text mode cursor movement is limited
            // For now, we'll just redraw the line
            self.redraw_input_line();
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
            self.redraw_input_line();
        }
    }

    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
        self.redraw_input_line();
    }

    pub fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.input_buffer.len();
        self.redraw_input_line();
    }

    // Handle delete key (delete character at cursor, not before)
    pub fn handle_delete(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.input_buffer.remove(self.cursor_position);
            self.redraw_input_line();
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
            "clear" => {
                commands::clear();
            }
            "about" => commands::about(),
            "version" => commands::version(),
            "hello" => commands::hello(),
            "shutdown" => commands::shutdown(),
            "express" => commands::express(),
            "time" => commands::time_test(),
            "read" => commands::read(),
            "run" => commands::run(),
            "eclipse" => {
                print_message("Eclipse OS - A modern operating system");
                print_message("Version: 0.1.1");
                print_message("Built with Rust");
            }
            _ => {
                play_melody(Melody::Error);
                print_message(&format!("Unknown command: '{command}'"));
                print_message("Type 'help' for available commands.");
            }
        }
    }
}

// Commands module updated for UEFI text buffer
pub mod commands {
    use alloc::format;

    use crate::rtc;
    use crate::text_editor::express_editor;
    use crate::uefi_text_buffer::{clear_output, print_message};

    pub fn help() {
        print_message("Available commands:");
        print_message("  help     - Show this help message");
        print_message("  echo     - Echo text back");
        print_message("  clear    - Clear the screen");
        print_message("  about    - About Eclipse OS");
        print_message("  version  - Show version information");
        print_message("  hello    - Say hello");
        print_message("  eclipse  - Show Eclipse OS info");
        print_message("  express  - Launch Express Editor");
        print_message("  shutdown - Shutdown the system");
    }

    pub fn echo(args: core::str::SplitWhitespace) {
        let text: alloc::vec::Vec<&str> = args.collect();
        print_message(&text.join(" "));
    }

    pub fn clear() {
        clear_output();
    }

    pub fn about() {
        print_message("Eclipse OS - A hobby operating system written in Rust");
        print_message("Features:");
        print_message("  - UEFI text mode");
        print_message("  - Keyboard input");
        print_message("  - Basic shell");
        print_message("  - Memory management");
        print_message("  - Express Editor");
    }

    pub fn version() {
        print_message("Eclipse OS version 0.1.0");
        print_message("Rust version: 1.0.0");
    }

    pub fn hello() {
        print_message("Hello from Eclipse OS!");
    }

    pub fn express() {
        print_message("Launching Express Editor...");
        express_editor::init_editor();
    }

    pub fn shutdown() {
        print_message("Shutting down Eclipse OS...");
        crate::hlt_loop();
    }

    pub fn time_test() {
        use crate::time;

        print_message(&format!("Current time: {}", rtc::get_current_time()));
        print_message(&format!("Current ticks: {}", time::get_ticks()));
        print_message(&format!("Time (ms): {}", time::get_time_ms()));
        print_message(&format!("Time (ns): {}", time::get_time_ns()));

        print_message(&format!("Uptime: {}", time::get_uptime_seconds()));

        if let Some(cpu_freq) = time::get_cpu_frequency_hz() {
            print_message(&format!("CPU frequency: {cpu_freq} Hz"));
        }
    }

    pub fn read() {
        use crate::crude_storage;

        crude_storage::read();
    }

    pub fn run() {
        use crate::crude_storage;

        crude_storage::run();
    }
}
