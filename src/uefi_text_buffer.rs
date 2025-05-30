use uefi::CStr16;
use crate::BootInfo;

pub static mut CURSOR_Y: usize = 0;

pub fn clear_output(boot_info: &BootInfo) {
    if let Some(mutex) = boot_info.text_output.get() {
        let mut output = mutex.lock();

        output.clear().expect("Failed to clear output!");

        unsafe { CURSOR_Y = 0 };
    }
}

// Function to boot to the uefi text mode
pub fn print_message(boot_info: &BootInfo, msg: &str) {
    if let Some(mutex) = boot_info.text_output.get() {
        let mut output = mutex.lock();

        unsafe {
            // Align left
            output.set_cursor_position(0, CURSOR_Y).expect("Failed to set cursor!");
            CURSOR_Y += 1;
        }
        
        let mut buf = [0u16; 128];
        let cstr16 = CStr16::from_str_with_buf(msg, &mut buf).unwrap();
        output.output_string(cstr16).unwrap();
    }
}