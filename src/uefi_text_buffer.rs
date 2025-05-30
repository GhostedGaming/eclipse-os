use uefi::CStr16;
use crate::{BootInfo, TEXT_OUTPUT};

pub static mut CURSOR_Y: usize = 0;
pub static mut CURSOR_X: usize = 0;

pub fn clear_output() {
    if let Some(mutex) = TEXT_OUTPUT.get() {
        let output = mutex.lock();
        let output_ptr = output.0;

        unsafe {
            output_ptr.as_mut().expect("Output pointer is null").clear().expect("Failed to clear output!");
            CURSOR_Y = 0;
            CURSOR_X = 0;
        }
    }
}

// Function to print a full message (moves to new line)
pub fn print_message(msg: &str) {
    if let Some(mutex) = TEXT_OUTPUT.get() {
        let output: spin::MutexGuard<'_, crate::OutputForced> = mutex.lock();
        let output_ptr = output.0;

        unsafe {
            // Set cursor position
            if let Some(text_output) = output_ptr.as_mut() {
                text_output.set_cursor_position(CURSOR_X, CURSOR_Y).expect("Failed to set cursor!");
            } else {
                panic!("Failed to get mutable reference to text output!");
            }
        }
        
        let mut buf = [0u16; 128];
        let cstr16 = CStr16::from_str_with_buf(msg, &mut buf).unwrap();
        unsafe { 
            output_ptr.as_mut().unwrap().output_string(cstr16).unwrap();
            // Move to next line after printing message
            CURSOR_Y += 1;
            CURSOR_X = 0;
        }
    }
}

// Function to print a single character (stays on same line)
pub fn print_char(ch: char) {
    if let Some(mutex) = TEXT_OUTPUT.get() {
        let output: spin::MutexGuard<'_, crate::OutputForced> = mutex.lock();
        let output_ptr = output.0;

        unsafe {
            // Set cursor position
            if let Some(text_output) = output_ptr.as_mut() {
                text_output.set_cursor_position(CURSOR_X, CURSOR_Y).expect("Failed to set cursor!");
            } else {
                panic!("Failed to get mutable reference to text output!");
            }
        }
        
        let char_str = alloc::string::ToString::to_string(&ch);
        let mut buf = [0u16; 8];
        let cstr16 = CStr16::from_str_with_buf(&char_str, &mut buf).unwrap();
        
        unsafe { 
            output_ptr.as_mut().unwrap().output_string(cstr16).unwrap();
            
            // Handle cursor movement
            match ch {
                '\n' => {
                    CURSOR_Y += 1;
                    CURSOR_X = 0;
                }
                '\r' => {
                    CURSOR_X = 0;
                }
                _ => {
                    CURSOR_X += 1;
                    // If we reach the end of the line, wrap to next line
                    // Assuming 80 columns (adjust as needed for your display)
                    if CURSOR_X >= 80 {
                        CURSOR_Y += 1;
                        CURSOR_X = 0;
                    }
                }
            }
        }
    }
}

// Function to handle backspace
pub fn backspace() {
    unsafe {
        if CURSOR_X > 0 {
            CURSOR_X -= 1;
            
            if let Some(mutex) = TEXT_OUTPUT.get() {
                let output = mutex.lock();
                let output_ptr = output.0;
                
                // Set cursor position
                if let Some(text_output) = output_ptr.as_mut() {
                    text_output.set_cursor_position(CURSOR_X, CURSOR_Y).expect("Failed to set cursor!");
                }
                
                // Print a space to erase the character
                let mut buf = [0u16; 2];
                let cstr16 = CStr16::from_str_with_buf(" ", &mut buf).unwrap();
                output_ptr.as_mut().unwrap().output_string(cstr16).unwrap();
                
                // Move cursor back
                if let Some(text_output) = output_ptr.as_mut() {
                    text_output.set_cursor_position(CURSOR_X, CURSOR_Y).expect("Failed to set cursor!");
                }
            }
        } else if CURSOR_Y > 0 {
            // Move to end of previous line
            CURSOR_Y -= 1;
            CURSOR_X = 79; // Assuming 80 columns, adjust as needed
        }
    }
}

// Function to get current cursor position
pub fn get_cursor_position() -> (usize, usize) {
    unsafe { (CURSOR_Y, CURSOR_X) }
}

// Function to set cursor position
pub fn set_cursor_position(row: usize, col: usize) {
    unsafe {
        CURSOR_Y = row;
        CURSOR_X = col;
        
        if let Some(mutex) = TEXT_OUTPUT.get() {
            let output = mutex.lock();
            let output_ptr = output.0;
            
            if let Some(text_output) = output_ptr.as_mut() {
                text_output.set_cursor_position(CURSOR_X, CURSOR_Y).expect("Failed to set cursor!");
            }
        }
    }
}