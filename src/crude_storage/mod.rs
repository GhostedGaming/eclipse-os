use crate::{intereperter::run, text_editor::express_editor::EDITOR_DATA, uefi_text_buffer::print_message};
use alloc::{
    string::{String, ToString},
    vec::Vec,
    format
};
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref ENTRIES: Mutex<Vec<Information>> = Mutex::new(Vec::new());
}

#[derive(Debug)]
pub struct Information {
    name: String,
    content: String,
}

pub fn write(input: String) {
    let mut entries = ENTRIES.lock();
    entries.push(Information {
        name: "Hello".to_string(),
        content: input,
    });
}

// TODO: make not dog shit
pub fn read() {
    let entries = ENTRIES.lock();
    let mut _content: String = "null".to_string();
    for entry in entries.iter() {
        print_message(&format!("Name: {}", entry.name));
        print_message(&format!("Conent: {}", entry.content));
        _content = entry.content.clone();
    }
}

pub fn read_no_print() -> String {
    let entries = ENTRIES.lock();
    let mut content: String = "null".to_string();
    for entry in entries.iter() {
        content = entry.content.clone();
    }

    content
}

pub fn run() {
    let content = read_no_print();
    if !content.trim().is_empty() && content != "null" {
        // Then set the content
        {
            let mut editor_data = EDITOR_DATA.lock();
            editor_data.text = content;
            editor_data.update_lines();
        }

        run::run_example();
    } else {
        print_message("No code stored to execute.");
    }
}
