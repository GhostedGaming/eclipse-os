use alloc::{string::{String, ToString}, vec::Vec};
use lazy_static::lazy_static;
use spin::Mutex;
use crate::{intereperter::run, shell, text_editor::{self, express_editor::EDITOR_DATA}};

use crate::println;

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

pub fn read() {
    let entries = ENTRIES.lock();
    let mut content: String = "null".to_string();
    for entry in entries.iter() {
        println!("Name: {}", entry.name);
        println!("Conent: {}", entry.content);
        content = entry.content.clone();
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
        println!("No code stored to execute.");
    }
}