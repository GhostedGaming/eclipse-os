use alloc::string::String;
use alloc::vec::Vec;
use crate::{print, println, vga_buffer};
extern crate alloc;
use lazy_static::lazy_static;
use spin::Mutex;

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
    fn update_lines(&mut self) {
        use alloc::string::ToString;
        self.lines = self.text.split('\n').map(|s| s.to_string()).collect();
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
            self.cursor_x -= 1;
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
    // Just return the buffer for the interpreter to process
    EDITOR_DATA.lock().text.clone()
}

/// Optionally process text (not required for basic input)
pub fn text_processor(text: String) -> String {
    text
}

/// Initializes the editor and clears the buffer.
pub fn init_editor() {
    let mut editor_data = EDITOR_DATA.lock();
    editor_data.active = true;
    editor_data.text.clear();
    editor_data.cursor = 0;
    editor_data.cursor_x = 0;
    editor_data.cursor_y = 0;
    editor_data.lines = Vec::new();
    
    vga_buffer::clear_screen();
    vga_buffer::set_cursor_visibility(true);
    vga_buffer::set_cursor_style(vga_buffer::CursorStyle::Block);
    
    println!("-- EXPRESS EDITOR --");
    println!("(Ctrl+C to exit)");
}

/// Setup function for any initialization needed
pub fn init_setup() {
    // Any one-time setup code goes here
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
        // Handle moving left at the beginning of a line
        if editor_data.cursor_x == 0 && editor_data.cursor_y > 0 {
            // Move to the end of the previous line
            editor_data.cursor_y -= 1;
            editor_data.cursor_x = editor_data.lines[editor_data.cursor_y].len();
        } else if editor_data.cursor_x > 0 {
            editor_data.cursor_x -= 1;
        }
        
        editor_data.cursor -= 1;
        vga_buffer::move_cursor_left();
    }
}

/// Move cursor right
pub fn move_cursor_right() {
    let mut editor_data = EDITOR_DATA.lock();
    if editor_data.active && editor_data.cursor < editor_data.text.len() {
        let current_line_length = editor_data.get_current_line_length();
        
        // Handle moving right at the end of a line
        if editor_data.cursor_x >= current_line_length && 
            editor_data.cursor_y < editor_data.lines.len() - 1 {
            // Move to the beginning of the next line
            editor_data.cursor_y += 1;
            editor_data.cursor_x = 0;
        } else if editor_data.cursor_x < current_line_length {
            editor_data.cursor_x += 1;
        }
        
        editor_data.cursor += 1;
        vga_buffer::move_cursor_right();
    }
}

/// Move cursor up
pub fn move_cursor_up() {
    let mut editor_data = EDITOR_DATA.lock();
    if editor_data.active && editor_data.cursor_y > 0 {
        // Calculate current position in text
        let current_line_offset = editor_data.text[..editor_data.cursor]
            .rfind('\n')
            .map_or(0, |pos| pos + 1);
            
        // Calculate previous line length
        editor_data.cursor_y -= 1;
        let prev_line_length = editor_data.lines[editor_data.cursor_y].len();
        
        // Adjust x position if needed
        if editor_data.cursor_x > prev_line_length {
            editor_data.cursor_x = prev_line_length;
        }
        
        // Update absolute cursor position
        editor_data.cursor = current_line_offset - editor_data.cursor_x - 1;
        
        // Update visual cursor
        // This would require more complex VGA buffer manipulation
        // For now, we'd need to redraw the screen
        redraw_editor();
    }
}

/// Move cursor down
pub fn move_cursor_down() {
    let mut editor_data = EDITOR_DATA.lock();
    if editor_data.active && editor_data.cursor_y < editor_data.lines.len() - 1 {
        // Calculate next line length
        editor_data.cursor_y += 1;
        let next_line_length = editor_data.lines[editor_data.cursor_y].len();
        
        // Adjust x position if needed
        if editor_data.cursor_x > next_line_length {
            editor_data.cursor_x = next_line_length;
        }
        
        // Calculate new absolute cursor position
        let mut pos = 0;
        for i in 0..editor_data.cursor_y {
            pos += editor_data.lines[i].len() + 1; // +1 for newline
        }
        pos += editor_data.cursor_x;
        editor_data.cursor = pos;
        
        // Update visual cursor
        // This would require more complex VGA buffer manipulation
        // For now, we'd need to redraw the screen
        redraw_editor();
    }
}

/// Redraw the editor content
fn redraw_editor() {
    let editor_data = EDITOR_DATA.lock();
    
    // Save cursor state
    let cursor_x = editor_data.cursor_x;
    let cursor_y = editor_data.cursor_y;
    
    // Clear screen and redraw content
    vga_buffer::clear_screen();
    println!("-- EXPRESS EDITOR --");
    println!("(Ctrl+C to exit)");
    
    // Print all lines
    for line in &editor_data.lines {
        println!("{}", line);
    }
    
    // Restore cursor position
    // This would require more complex VGA buffer manipulation
    // For a simple implementation, we'd need to calculate screen position
}

/// Exits the editor, resets state, and runs the interpreter.
pub fn exit_editor() {
    {
        let mut editor_data = EDITOR_DATA.lock();
        editor_data.active = false;
        vga_buffer::clear_screen();
        vga_buffer::set_cursor_visibility(true);
        vga_buffer::set_color(vga_buffer::Color::White, vga_buffer::Color::Black);
        println!("-- Exited Express Editor --");
        // Do NOT clear editor_data.text here, so interpreter can read it!
    } // lock dropped
    
    crate::intereperter::run::run_example();
    
    // Now clear the buffer after running the interpreter
    {
        let mut editor_data = EDITOR_DATA.lock();
        editor_data.text.clear();
        editor_data.cursor = 0;
        editor_data.cursor_x = 0;
        editor_data.cursor_y = 0;
        editor_data.lines.clear();
    }
}
