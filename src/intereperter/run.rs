extern crate alloc;
extern crate libm;

use crate::intereperter::components::env;
use crate::intereperter::components::ops;
use crate::intereperter::components::parser;
use crate::intereperter::components::tokens;
use crate::text_editor::express_editor::get_text;
use crate::{println};
use alloc::string::String;
use alloc::vec::Vec;

pub use env::*;
pub use ops::*;
pub use parser::*;
pub use tokens::*;

// Parser for expressions with precedence

pub fn run_example() {
    let input = get_text();

    // Reset the global environment
    *GLOBAL_ENV.lock() = Environment::new();

    // Process the entire input as a single block of code
    let mut code_blocks: Vec<String> = Vec::new();
    let mut current_block = String::new();
    let mut brace_count = 0;

    // Split the input into lines
    let lines: Vec<&str> = input.lines().collect();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        // Count braces to track code blocks
        for ch in trimmed.chars() {
            if ch == '{' {
                brace_count += 1;
            } else if ch == '}' {
                brace_count -= 1;
            }
        }

        if !current_block.is_empty() {
            current_block.push(' ');
        }
        current_block.push_str(trimmed);

        // If we're not inside a block or we've completed a block, evaluate it
        if brace_count == 0 && !current_block.is_empty() {
            if current_block.ends_with(';')
                || current_block.ends_with('}')
                || !current_block.contains('{')
            {
                code_blocks.push(current_block.clone());
                current_block.clear();
            }
        }
    }

    // Add any remaining code
    if !current_block.is_empty() {
        code_blocks.push(current_block);
    }

    // Evaluate each code block
    for block in code_blocks {
        // Tokenize and parse the block
        let tokens = lexer(&block);
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
    }
}

pub fn evaluate(input: &str) -> f64 {
    let tokens = lexer(input);
    let mut parser = Parser::new(tokens);
    parser.parse()
}
