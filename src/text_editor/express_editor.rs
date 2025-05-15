use alloc::string::String;
use crate::println;
extern crate alloc;
use lazy_static::lazy_static;
use spin::Mutex;

/// A struct to represent editor data.
pub struct Data {
    pub active: bool,
    pub text: String,
    pub cursor: usize,
}

lazy_static! {
    pub static ref EDITOR_DATA: Mutex<Data> = Mutex::new(Data {
        active: false,
        text: String::new(),
        cursor: 0,
    });
}

/// Returns the code currently in the editor buffer as a String.
pub fn test() -> String {
    // Just return the buffer for the interpreter to process
    EDITOR_DATA.lock().text.clone()
}

/// Optionally process text (not required for basic input)
pub fn text_processor(text: String) -> String {
    text.to_uppercase()
}

/// Initializes the editor and clears the buffer.
pub fn init_editor() {
    let mut editor_data = EDITOR_DATA.lock();
    editor_data.active = true;
    editor_data.text.clear();
    editor_data.cursor = 0;
    crate::vga_buffer::clear_screen();
    crate::vga_buffer::set_cursor_visibility(true);
    crate::vga_buffer::set_cursor_style(crate::vga_buffer::CursorStyle::Block);
    println!("-- EXPRESS EDITOR --");
    println!("(Ctrl+C to exit)");
}

pub fn init_setup() {
    
}

/// Handles a single character input in the editor.
pub fn process_editor_key(c: char) {
    let mut editor_data = EDITOR_DATA.lock();
    if c == '\u{8}' {
        // Backspace
        if !editor_data.text.is_empty() {
            editor_data.text.pop();
            crate::vga_buffer::backspace();
        }
    } else if c == '\n' {
        editor_data.text.push('\n');
        println!();
    } else if c.is_ascii() && !c.is_control() {
        editor_data.text.push(c);
        crate::print!("{}", c);
    }
}

/// Exits the editor, resets state, and runs the interpreter.
pub fn exit_editor() {
    {
        let mut editor_data = EDITOR_DATA.lock();
        editor_data.active = false;
        crate::vga_buffer::clear_screen();
        crate::vga_buffer::set_cursor_visibility(true);
        crate::vga_buffer::set_color(crate::vga_buffer::Color::White, crate::vga_buffer::Color::Black);
        crate::println!("-- Exited Express Editor --");
        // Do NOT clear editor_data.text here, so interpreter can read it!
    } // lock dropped

    crate::intereperter::run::run_example();

    // Now clear the buffer after running the interpreter
    {
        let mut editor_data = EDITOR_DATA.lock();
        editor_data.text.clear();
    }
}