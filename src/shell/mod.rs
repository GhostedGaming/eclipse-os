use crate::{print, println, vga_buffer};
use crate::pc_speaker::{play_melody, Melody};
use alloc::string::String;

pub struct Shell {
    input_buffer: String,
    cursor_position: usize, 
    prompt: &'static str,
    prompt_row: usize,      
    prompt_col: usize,      
    input_start_row: usize, 
    input_start_col: usize, 
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            input_buffer: String::new(),
            cursor_position: 0,
            prompt: "eclipse-os> ",
            prompt_row: 0,
            prompt_col: 0,
            input_start_row: 0,
            input_start_col: 0,
        }
    }

    pub fn start(&mut self) {
        self.show_prompt();
    }

    fn show_prompt(&mut self) {
        // Get current cursor position before showing prompt
        let (row, col) = vga_buffer::get_cursor_position();
        print!("{}", self.prompt);

        // Update our tracking of where the prompt and input area are
        self.prompt_row = row;
        self.prompt_col = col;
        self.input_start_row = row;
        self.input_start_col = col + self.prompt.len();
    }

    pub fn process_keypress(&mut self, key: char) {
        match key {
            '\n' => {
                println!(); // Move to next line
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
                print!("{}", c);
            } else {
                // Need to redraw if inserting in the middle
                self.redraw_input_line();
            }
        } else {
            // Fallback: just append and print
            self.input_buffer.push(c);
            self.cursor_position = self.input_buffer.len();
            print!("{}", c);
        }
    }

    fn handle_backspace(&mut self) {
        // Only allow backspace if we have input to remove
        if self.cursor_position > 0 && !self.input_buffer.is_empty() {
            // Check if we're at the protected boundary
            let (current_row, current_col) = vga_buffer::get_cursor_position();

            // Don't allow backspace if it would go into the prompt area
            if current_row < self.input_start_row
                || (current_row == self.input_start_row && current_col <= self.input_start_col)
            {
                // Play beep sound to indicate we can't backspace further
                crate::pc_speaker::beep(500, 10);
                return;
            }

            // Remove character before cursor
            self.cursor_position -= 1;
            self.input_buffer.remove(self.cursor_position);

            // For simple backspace at the end, just use VGA backspace
            if self.cursor_position == self.input_buffer.len() {
                // Only backspace if we're in the input area
                let (row, col) = vga_buffer::get_cursor_position();
                if row > self.input_start_row
                    || (row == self.input_start_row && col > self.input_start_col)
                {
                    vga_buffer::backspace();
                }
            } else {
                // Need to redraw if removing from the middle
                self.redraw_input_line();
            }
        } else {
            // At beginning of input, just make a beep sound
            crate::pc_speaker::beep(500, 10);
        }
    }

    fn redraw_input_line(&mut self) {
        // Save current cursor position
        let (current_row, current_col) = vga_buffer::get_cursor_position();

        // Move to start of input area (NOT the prompt)
        vga_buffer::set_cursor_position(self.input_start_row, self.input_start_col);

        // Clear only the input area by printing spaces
        let max_input_length = vga_buffer::BUFFER_WIDTH - self.input_start_col;
        for _ in 0..max_input_length {
            print!(" ");
        }

        // Move back to start of input area
        vga_buffer::set_cursor_position(self.input_start_row, self.input_start_col);

        // Print the current input buffer
        print!("{}", self.input_buffer);

        // Position cursor correctly within the input
        let target_col = self.input_start_col + self.cursor_position;
        if target_col < vga_buffer::BUFFER_WIDTH {
            vga_buffer::set_cursor_position(self.input_start_row, target_col);
        }
    }

    // Add methods to handle cursor movement within the input
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            let (current_row, current_col) = vga_buffer::get_cursor_position();

            // Don't move left if it would go into the prompt
            if current_row == self.input_start_row && current_col <= self.input_start_col {
                return;
            }

            self.cursor_position -= 1;
            vga_buffer::move_cursor_left(1);
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
            vga_buffer::move_cursor_right(1);
        }
    }

    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
        vga_buffer::set_cursor_position(self.input_start_row, self.input_start_col);
    }

    pub fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.input_buffer.len();
        let target_col = self.input_start_col + self.cursor_position;
        vga_buffer::set_cursor_position(self.input_start_row, target_col);
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
                // After clearing, we need to reset our position tracking
                // since clear_screen resets cursor to (0,0)
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
                println!("Eclipse OS - A modern operating system");
                println!("Version: 0.1.1");
                println!("Built with Rust");
            }
            _ => {
                play_melody(Melody::Error);
                println!("Unknown command: '{}'", command);
                println!("Type 'help' for available commands.");
            }
        }
    }
}

// Commands module remains the same
pub mod commands {
    use crate::text_editor::express_editor;
    use crate::{println, rtc, vga_buffer};

    pub fn help() {
        println!("Available commands:");
        println!("  help     - Show this help message");
        println!("  echo     - Echo text back");
        println!("  clear    - Clear the screen");
        println!("  about    - About Eclipse OS");
        println!("  version  - Show version information");
        println!("  hello    - Say hello");
        println!("  eclipse  - Show Eclipse OS info");
        println!("  express  - Launch Express Editor");
        println!("  shutdown - Shutdown the system");
    }

    pub fn echo(args: core::str::SplitWhitespace) {
        let text: alloc::vec::Vec<&str> = args.collect();
        println!("{}", text.join(" "));
    }

    pub fn clear() {
        vga_buffer::clear_screen();
    }

    pub fn about() {
        println!("Eclipse OS - A hobby operating system written in Rust");
        println!("Features:");
        println!("  - VGA text mode");
        println!("  - Keyboard input");
        println!("  - Basic shell");
        println!("  - Memory management");
        println!("  - Express Editor");
    }

    pub fn version() {
        println!("Eclipse OS version 0.1.0");
        println!("Rust version: 1.0.0");
    }

    pub fn hello() {
        println!("Hello from Eclipse OS!");
    }

    pub fn express() {
        println!("Launching Express Editor...");
        express_editor::init_editor();
    }

    pub fn shutdown() {
        println!("Shutting down Eclipse OS...");
        crate::hlt_loop();
    }

    pub fn time_test() {
        use crate::time;
        
        println!("Current time: {}", rtc::get_current_time());
        println!("Current ticks: {}", time::get_ticks());
        println!("Time (ms): {}", time::get_time_ms());
        println!("Time (ns): {}", time::get_time_ns());

        println!("Uptime: {}", time::get_uptime_seconds());

        // if let Some(precise_ns) = time::get_precise_time_ns() {
        //     println!("Precise time (ns): {}", precise_ns);
        // }

        if let Some(cpu_freq) = time::get_cpu_frequency_hz() {
            println!("CPU frequency: {} Hz", cpu_freq);
        }
    }

    pub fn read() {
        use crate::crude_storage::crude_storage;

        crude_storage::read();
    }

    pub fn run() {
        use crate::crude_storage::crude_storage; // Rhymes with grug

        crude_storage::run();
    }
}