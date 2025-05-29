use uefi::CStr16;
use crate::BootInfo;

pub fn print_message(boot_info: &BootInfo, msg: &str) {
    if let Some(mutex) = boot_info.text_output.get() {
        let mut output = mutex.lock();
        output.clear().expect("Failed to clear screen");

        let mut buf = [0u16; 128];
        let cstr16 = CStr16::from_str_with_buf(msg, &mut buf).unwrap();
        output.output_string(cstr16).unwrap();
    }
}