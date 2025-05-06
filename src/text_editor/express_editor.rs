use alloc::string::String;
use crate::println;
extern crate alloc;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::fs;
use crate::task::keyboard;
use crate::alloc::string::ToString;

/// A struct to represent editor data.
pub struct Data {
    pub active: bool,
    pub text: String,
}

lazy_static! {
    pub static ref EDITOR_DATA: Mutex<Data> = Mutex::new(Data {
        active: false,
        text: String::new(),
    });
}

pub fn test() -> String {
    // Set some initial text
    EDITOR_DATA.lock().text = 
    r"205+255".to_string();

    // Initialize the editor (this will process the text)
    init_editor();

    // Check the processed text
    let processed_text = EDITOR_DATA.lock().text.clone();
    println!("Processed Text: {}", processed_text);
    return EDITOR_DATA.lock().text.to_string();
}

pub fn text_processor(text: String) -> String {
    text.to_uppercase()
}

pub fn init_editor() {
    let mut editor_data = EDITOR_DATA.lock();
    editor_data.active = true;
    println!("Welcome to the Text Editor!");
}

pub fn init_setup() {
    
}

pub fn exit_editor() {
    if EDITOR_DATA.lock().active {
        EDITOR_DATA.lock().active = false;
        println!("Exiting editors...");
    }
}