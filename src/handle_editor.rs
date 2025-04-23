use crate::express::express_editor;
use crate::vga_buffer;

pub struct Data {
    active: bool,
    text: &'static str, // Use a static string slice instead of String
}

impl Data {
    // A method to activate the `active` field
    pub fn set_active(&mut self) {
        self.active = true;
    }
}

fn handle_editor() {
    let mut data = Data {
        active: false,
        text: "Initial text", // Static string slice
    };

    data.set_active(); // Set `active` to true

    // Send output to VGA buffer (or a similar mechanism for no_std environments)
    vga_buffer::print(format_args!("Is active: {}, Text: {}", data.active, data.text));
}