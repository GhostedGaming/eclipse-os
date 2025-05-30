use uefi::CStr16;
use crate::{BootInfo, TEXT_OUTPUT};

pub static mut CURSOR_Y: usize = 0;

pub fn clear_output() {
    if let Some(mutex) = TEXT_OUTPUT.get() {
        let output = mutex.lock();
        let output_ptr = output.0;

        unsafe {
            output_ptr.as_mut().expect("Output pointer is null").clear().expect("Failed to clear output!");
            CURSOR_Y = 0;
        }
    }
}

// Function to boot to the uefi text mode
pub fn print_message(msg: &str) {
    if let Some(mutex) = TEXT_OUTPUT.get() {
        let output = mutex.lock();
        let output_ptr = output.0;

        unsafe {
            // Align left
            if let Some(text_output) = output_ptr.as_mut() {
                text_output.set_cursor_position(0, CURSOR_Y).expect("Failed to set cursor!");
            } else {
                panic!("Failed to get mutable reference to text output!");
            }
            CURSOR_Y += 1;
        }
        
        let mut buf = [0u16; 128];
        let cstr16 = CStr16::from_str_with_buf(msg, &mut buf).unwrap();
        unsafe { output_ptr.as_mut().unwrap().output_string(cstr16).unwrap() };
    }
}