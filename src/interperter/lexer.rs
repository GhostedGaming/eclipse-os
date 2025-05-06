#![no_std]
extern crate alloc;
extern crate libm;

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
    Power,
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
            '^' => tokens.push(Tokens::Power),
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
    if right == 0.0 || right == -0.0 || right == f64::INFINITY || right == f64::NEG_INFINITY {
        panic!("Error connot divide by zero!");
    } else {
        return left / right
    }
}

fn power(left: f64, right: f64) -> f64 {
    return libm::pow(left, right);
}

pub fn run_example() {
    let mut result: f64 = 0.0;
    let input = test();
    let tokens = lexer(&input);
    
    // Simulate calculation with proper order
    let mut iter = tokens.into_iter();
    
    // Get the left operand
    let left = match iter.next() {
        Some(Tokens::Number(value)) => value,
        _ => panic!("Expected a number for left operand"),
    };
    
    // Get the operator
    let operator = match iter.next() {
        Some(op @ Tokens::Plus) => op,
        Some(op @ Tokens::Minus) => op,
        Some(op @ Tokens::Multiply) => op,
        Some(op @ Tokens::Divide) => op,
        Some(op @ Tokens::Power) => op,
        _ => panic!("Expected an operator (+, -, *, /)"),
    };
    
    // Get the right operand
    let right = match iter.next() {
        Some(Tokens::Number(value)) => value,
        _ => panic!("Expected a number for right operand"),
    };
    
    // Perform the calculation based on the operator
    result = match operator {
        Tokens::Plus => add(left, right),
        Tokens::Minus => substract(left, right),
        Tokens::Multiply => multiply(left, right),
        Tokens::Divide => divide(left, right),
        Tokens::Power => power(left, right),
        _ => unreachable!(), // We've already checked for valid operators
    };
    
    println!("Result: {}", result);
}