use alloc::string::String;
use crate::println;
extern crate alloc;
use lazy_static::lazy_static;
use spin::Mutex;

/// A struct to represent editor data.
pub struct Data {
    pub active: bool,
    pub text: String,
}

lazy_static! {
    static ref EDITOR_DATA: Mutex<Data> = Mutex::new(Data {
        active: false,
        text: String::new(),
    });
}

impl Data {
    pub fn set_express_editor(&mut self, state: bool) {
        self.active = state;
    }
}

pub fn set_editor_active(state: bool) {
    EDITOR_DATA.lock().active = state;
}

pub fn text_processor(text: String) -> String {
    text.to_uppercase()
}

pub fn init_editor() {
    println!("Welcome to the Text Editor!");
}

pub fn init_setup() {

}
