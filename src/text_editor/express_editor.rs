use alloc::string::String;
use alloc::vec::Vec;
use crate::{print, println, vga_buffer};
extern crate alloc;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::crude_storage::crude_storage;

/// A struct to represent editor data.
pub struct Data {
    pub active: bool,
    pub text: String,
    pub cursor: usize,      // Absolute position in text
    pub cursor_x: usize,    // Column position
    pub cursor_y: usize,    // Row position
    pub lines: Vec<String>, // Text split into lines for easier editing
}

impl Data {
    /// Gets the length of the current line where the cursor is positioned
    pub fn get_current_line_length(&self) -> usize {
        if self.cursor_y < self.lines.len() {
            self.lines[self.cursor_y].len()
        } else {
            0
        }
    }

    /// Updates the lines vector based on the text content
    pub fn update_lines(&mut self) {
        use alloc::string::ToString;
        self.lines = self.text.split('\n').map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
    }

    /// Inserts a character at the current cursor position
    pub fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += 1;
        self.cursor_x += 1;
        self.update_lines();
    }

    /// Removes a character before the cursor position
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.text.remove(self.cursor - 1);
            self.cursor -= 1;
            if self.cursor_x > 0 {
                self.cursor_x -= 1;
            } else if self.cursor_y > 0 {
                // Moving to previous line
                self.cursor_y -= 1;
                self.cursor_x = self.lines[self.cursor_y].len();
            }
            self.update_lines();
        }
    }

    /// Inserts a newline at the current cursor position
    pub fn insert_newline(&mut self) {
        self.text.insert(self.cursor, '\n');
        self.cursor += 1;
        self.cursor_y += 1;
        self.cursor_x = 0;
        self.update_lines();
    }
}

lazy_static! {
    pub static ref EDITOR_DATA: Mutex<Data> = Mutex::new(Data {
        active: false,
        text: String::new(),
        cursor: 0,
        cursor_x: 0,
        cursor_y: 0,
        lines: Vec::new(),
    });
}

/// Returns the code currently in the editor buffer as a String.
pub fn get_text() -> String {
    EDITOR_DATA.lock().text.clone()
}

/// Initializes the editor and clears the buffer.
pub fn init_editor() {
    {
        let mut editor_data = EDITOR_DATA.lock();
        editor_data.active = true;
        editor_data.text.clear();
        editor_data.cursor = 0;
        editor_data.cursor_x = 0;
        editor_data.cursor_y = 0;
        editor_data.lines = Vec::new();
        editor_data.update_lines(); // Initialize with empty line
    }
    
    vga_buffer::clear_screen();
    vga_buffer::set_cursor_visibility(true);
    vga_buffer::set_cursor_style(vga_buffer::CursorStyle::Block);
    
    println!("-- EXPRESS EDITOR --");
    println!("Type your code below. Press Ctrl+C to exit and run.\n");
    
    // Position cursor for input
    vga_buffer::set_cursor_position(3, 0);
}

/// Handles a single character input in the editor.
pub fn process_editor_key(c: char) {
    let mut editor_data = EDITOR_DATA.lock();
    
    match c {
        '\u{8}' => { // Backspace
            if !editor_data.text.is_empty() && editor_data.cursor > 0 {
                editor_data.backspace();
                vga_buffer::backspace();
            }
        },
        '\n' => {
            editor_data.insert_newline();
            println!();
        },
        c if c.is_ascii() && !c.is_control() => {
            editor_data.insert_char(c);
            print!("{}", c);
        },
        _ => {} // Ignore other characters
    }
}

/// Move cursor left
pub fn move_cursor_left() {
    let mut editor_data = EDITOR_DATA.lock();
    if editor_data.active && editor_data.cursor > 0 {
        if editor_data.cursor_x == 0 && editor_data.cursor_y > 0 {
            // Move to the end of the previous line
            editor_data.cursor_y -= 1;
            editor_data.cursor_x = editor_data.lines[editor_data.cursor_y].len();
        } else if editor_data.cursor_x > 0 {
            editor_data.cursor_x -= 1;
        }
        
        editor_data.cursor -= 1;
        vga_buffer::move_cursor_left(1);
    }
}

/// Move cursor right
pub fn move_cursor_right() {
    let mut editor_data = EDITOR_DATA.lock();
    if editor_data.active && editor_data.cursor < editor_data.text.len() {
        let current_line_length = editor_data.get_current_line_length();
        
        if editor_data.cursor_x >= current_line_length && 
            editor_data.cursor_y < editor_data.lines.len() - 1 {
            // Move to the beginning of the next line
            editor_data.cursor_y += 1;
            editor_data.cursor_x = 0;
        } else if editor_data.cursor_x < current_line_length {
            editor_data.cursor_x += 1;
        }
        
        editor_data.cursor += 1;
        vga_buffer::move_cursor_right(1);
    }
}

/// Move cursor up
pub fn move_cursor_up() {
    let mut editor_data = EDITOR_DATA.lock();
    if editor_data.active && editor_data.cursor_y > 0 {
        editor_data.cursor_y -= 1;
        let prev_line_length = editor_data.lines[editor_data.cursor_y].len();
        
        if editor_data.cursor_x > prev_line_length {
            editor_data.cursor_x = prev_line_length;
        }
        
        // Recalculate absolute cursor position
        let mut pos = 0;
        for i in 0..editor_data.cursor_y {
            pos += editor_data.lines[i].len() + 1; // +1 for newline
        }
        pos += editor_data.cursor_x;
        editor_data.cursor = pos;
        
        vga_buffer::move_cursor_up(1);
    }
}

/// Move cursor down
pub fn move_cursor_down() {
    let mut editor_data = EDITOR_DATA.lock();
    if editor_data.active && editor_data.cursor_y < editor_data.lines.len() - 1 {
        editor_data.cursor_y += 1;
        let next_line_length = editor_data.lines[editor_data.cursor_y].len();
        
        if editor_data.cursor_x > next_line_length {
            editor_data.cursor_x = next_line_length;
        }
        
        // Recalculate absolute cursor position
        let mut pos = 0;
        for i in 0..editor_data.cursor_y {
            pos += editor_data.lines[i].len() + 1; // +1 for newline
        }
        pos += editor_data.cursor_x;
        editor_data.cursor = pos;
        
        vga_buffer::move_cursor_down(1);
    }
}

/// Exits the editor, resets state, and runs the interpreter.
pub fn exit_editor() {
    let code_to_run = {
        let mut editor_data = EDITOR_DATA.lock();
        editor_data.active = false;
        let code = editor_data.text.clone();
        
        vga_buffer::clear_screen();
        vga_buffer::set_cursor_visibility(true);
        vga_buffer::set_color(vga_buffer::Color::White, vga_buffer::Color::Black);
        
        println!("-- Exited Express Editor --");
        println!("Running code:");
        println!("----------------------------------------");
        println!("{}", code);
        println!("----------------------------------------");

        crude_storage::write(code.clone());

        code
    }; // lock dropped here
    
    // Run the interpreter with the code
    if !code_to_run.trim().is_empty() {
        println!("Executing code...");
        
        // Actually call the interpreter here!
        crate::intereperter::run::run_example();
        
        println!("Code execution completed.");
    } else {
        println!("No code to execute.");
    }
    
    // Clear the buffer after running
    {
        let mut editor_data = EDITOR_DATA.lock();
        editor_data.text.clear();
        editor_data.cursor = 0;
        editor_data.cursor_x = 0;
        editor_data.cursor_y = 0;
        editor_data.lines.clear();
    }
    
    // Return to shell
    println!();
    crate::task::keyboard::init_shell();
}

/// Move cursor to start of current line
pub fn move_to_line_start() {
    let mut editor_data = EDITOR_DATA.lock();
    if editor_data.active {
        // Calculate how many characters to move back to reach line start
        let chars_to_move = editor_data.cursor_x;
        editor_data.cursor -= chars_to_move;
        editor_data.cursor_x = 0;
        
        // Move VGA cursor to start of line
        vga_buffer::move_cursor_to_start_of_line();
    }
}

/// Move cursor to end of current line
pub fn move_to_line_end() {
    let mut editor_data = EDITOR_DATA.lock();
    if editor_data.active && editor_data.cursor_y < editor_data.lines.len() {
        let current_line_length = editor_data.lines[editor_data.cursor_y].len();
        let chars_to_move = current_line_length - editor_data.cursor_x;
        
        editor_data.cursor += chars_to_move;
        editor_data.cursor_x = current_line_length;
        
        // Move VGA cursor to end of line content
        vga_buffer::move_cursor_right(chars_to_move);
    }
}
