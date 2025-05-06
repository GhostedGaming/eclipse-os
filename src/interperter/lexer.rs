#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use crate::text_editor::express_editor::test;
use crate::println;

#[derive(Debug, PartialEq)]
pub enum Tokens {
    Let,
    Fn,
    Identifier(String),
    Number(f64),
    Plus,
    Minus,
    Multiply,
    Divide,
    EOF,
}

fn lexer(src: &str) -> Vec<Tokens> {
    let mut tokens = Vec::new();
    let mut chars = src.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            ' ' | '\n' | '\t' => {
                // Skip whitespace
            }
            '+' => tokens.push(Tokens::Plus),
            '-' => tokens.push(Tokens::Minus),
            '*' => tokens.push(Tokens::Multiply),
            '/' => tokens.push(Tokens::Divide),
            '0'..='9' => {
                // Parse numbers
                let mut number = ch.to_string();
                while let Some(next_ch) = chars.peek() {
                    if next_ch.is_numeric() || *next_ch == '.' {
                        number.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                tokens.push(Tokens::Number(number.parse().unwrap()));
            }
            'a'..='z' | 'A'..='Z' => {
                // Parse identifiers or keywords
                let mut identifier = ch.to_string();
                while let Some(next_ch) = chars.peek() {
                    if next_ch.is_alphanumeric() || *next_ch == '_' {
                        identifier.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                match identifier.as_str() {
                    "let" => tokens.push(Tokens::Let),
                    "fn" => tokens.push(Tokens::Fn),
                    _ => tokens.push(Tokens::Identifier(identifier)),
                }
            }
            _ => {
                // Handle unexpected characters
            }
        }
    }

    tokens.push(Tokens::EOF);
    tokens
}

fn add(left: f64, right: f64) -> f64 {
    return left + right
}

fn substract(left: f64, right: f64) -> f64 {
    return left - right
}

fn multiply(left: f64, right: f64) -> f64 {
    return left * right
}

fn divide(left: f64, right: f64) -> f64 {
    if right == 0.0 {
        panic!("Error connot divide by zero!");
    } else {
        return left / right
    }
}

pub fn run_example() {
    let input = test();
    let tokens = lexer(&input);

    // Simulate addition
    let mut iter = tokens.into_iter();
    let left = match iter.next() {
        Some(Tokens::Number(value)) => value,
        _ => panic!("Expected a number"),
    };

    let operator = iter.next();
    if operator != Some(Tokens::Plus) {
        panic!("Expected a '+' operator");
    }

    let right = match iter.next() {
        Some(Tokens::Number(value)) => value,
        _ => panic!("Expected a number"),
    };

    let result = add(left, right);
    println!("Result: {}", result);
}