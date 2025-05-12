use alloc::string::String;
use crate::println;
extern crate alloc;
use lazy_static::lazy_static;
use spin::Mutex;
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
    // Set a simpler example to debug
    EDITOR_DATA.lock().text = r#"
// Variable assignment
let x = 10;
let y = 5;
x + y;

// If statement example
if x > y {
    println("Hello, world!");
}

fn add(a, b) {
    return a + b;
}

let result = add(5, 3);
println(result);
"#
.to_string();
    
    // Initialize the editor
    init_editor();
    
    // Return the text for the interpreter to process
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
    let mut editor_data = EDITOR_DATA.lock();

    if editor_data.active {
        println!("Condition is true, setting active to false");
        editor_data.active = false;
        println!("Exiting editors...");
    }
}