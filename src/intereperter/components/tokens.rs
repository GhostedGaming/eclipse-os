use alloc::{string::{String, ToString}, vec::Vec};

use crate::println;

#[derive(Debug, PartialEq, Clone)]
pub enum Tokens {
    Print,
    Println,
    Asm,
    Let,
    Fn,
    If,
    Else,
    While,
    Return,
    True,
    False,
    Increment,
    Decrement,
    Identifier(String),
    Number(f64),
    String(String),
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    Assign,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    And,
    Or,
    Not,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    Comma,
    EOF,
}

// Define a structure to represent a function
#[derive(Clone)]
struct Function {
    name: String,
    parameters: Vec<String>,
    body_tokens: Vec<Tokens>,
    body_start: usize,
    body_end: usize,
}

pub fn lexer(src: &str) -> Vec<Tokens> {
    let mut tokens = Vec::new();
    let mut chars = src.chars().peekable();

    if src.trim().starts_with("//") {
        return tokens; // Return empty tokens for comment lines
    }

    while let Some(ch) = chars.next() {
        match ch {
            ' ' | '\n' | '\t' => {
                // Skip whitespace
            }
            '/' => {
                if let Some(&'/') = chars.peek() {
                    // This is a comment, skip the rest of the line
                    break;
                } else {
                    tokens.push(Tokens::Divide);
                }
            }
            '+' => {
                if let Some(&'+') = chars.peek() {
                    chars.next(); // consume the second '+'
                    tokens.push(Tokens::Increment);
                } else {
                    tokens.push(Tokens::Plus);
                }
            }
            '-' => {
                if let Some(&'-') = chars.peek() {
                    chars.next(); // consume the second '-'
                    tokens.push(Tokens::Decrement);
                } else {
                    tokens.push(Tokens::Minus);
                }
            }
            '*' => tokens.push(Tokens::Multiply),
            '^' => tokens.push(Tokens::Power),
            '(' => tokens.push(Tokens::LeftParen),
            ')' => tokens.push(Tokens::RightParen),
            '{' => tokens.push(Tokens::LeftBrace),
            '}' => tokens.push(Tokens::RightBrace),
            ';' => tokens.push(Tokens::Semicolon),
            ',' => tokens.push(Tokens::Comma),
            '=' => {
                if let Some(&'=') = chars.peek() {
                    chars.next(); // consume the second '='
                    tokens.push(Tokens::Equal);
                } else {
                    tokens.push(Tokens::Assign);
                }
            }
            '!' => {
                if let Some(&'=') = chars.peek() {
                    chars.next(); // consume the '='
                    tokens.push(Tokens::NotEqual);
                } else {
                    tokens.push(Tokens::Not);
                }
            }
            '<' => {
                if let Some(&'=') = chars.peek() {
                    chars.next(); // consume the '='
                    tokens.push(Tokens::LessThanEqual);
                } else {
                    tokens.push(Tokens::LessThan);
                }
            }
            '>' => {
                if let Some(&'=') = chars.peek() {
                    chars.next(); // consume the '='
                    tokens.push(Tokens::GreaterThanEqual);
                } else {
                    tokens.push(Tokens::GreaterThan);
                }
            }
            '&' => {
                if let Some(&'&') = chars.peek() {
                    chars.next(); // consume the second '&'
                    tokens.push(Tokens::And);
                } else {
                    // Handle unexpected character
                    println!("Unexpected character: &");
                }
            }
            '|' => {
                if let Some(&'|') = chars.peek() {
                    chars.next(); // consume the second '|'
                    tokens.push(Tokens::Or);
                } else {
                    // Handle unexpected character
                    println!("Unexpected character: |");
                }
            }
            '"' => {
                // Parse string literals
                let mut string = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '"' {
                        chars.next(); // consume the closing quote
                        break;
                    }
                    string.push(chars.next().unwrap());
                }
                tokens.push(Tokens::String(string));
            }
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
            'a'..='z' | 'A'..='Z' | '_' => {
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
                    "if" => tokens.push(Tokens::If),
                    "else" => tokens.push(Tokens::Else),
                    "while" => tokens.push(Tokens::While),
                    "return" => tokens.push(Tokens::Return),
                    "true" => tokens.push(Tokens::True),
                    "false" => tokens.push(Tokens::False),
                    "print" => tokens.push(Tokens::Print),
                    "println" => tokens.push(Tokens::Println),
                    "asm" => tokens.push(Tokens::Asm),
                    _ => tokens.push(Tokens::Identifier(identifier)),
                }
            }
            _ => {
                // Handle unexpected characters
                println!("Unexpected character: {}", ch);
            }
        }
    }

    tokens.push(Tokens::EOF);
    tokens
}