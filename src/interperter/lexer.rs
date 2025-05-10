extern crate alloc;
extern crate libm;

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::collections::BTreeMap;
use crate::text_editor::express_editor::test;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::println;

#[derive(Debug, PartialEq, Clone)]
pub enum Tokens {
    Let,
    Fn,
    If,
    Else,
    While,
    Return,
    True,
    False,
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

fn lexer(src: &str) -> Vec<Tokens> {
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
            '+' => tokens.push(Tokens::Plus),
            '-' => tokens.push(Tokens::Minus),
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

// Basic operations
fn add(left: f64, right: f64) -> f64 {
    left + right
}

fn subtract(left: f64, right: f64) -> f64 {
    left - right
}

fn multiply(left: f64, right: f64) -> f64 {
    left * right
}

fn divide(left: f64, right: f64) -> f64 {
    if right == 0.0 || right == -0.0 {
        panic!("Cannot divide by 0");
    } else if right == f64::INFINITY || right == f64::NEG_INFINITY {
        panic!("Cannot divide by infinity");
    } else {
        left / right
    }
}

fn power(left: f64, right: f64) -> f64 {
    libm::pow(left, right)
}

// Comparison operations
fn equals(left: f64, right: f64) -> bool {
    (left - right).abs() < f64::EPSILON
}

fn not_equals(left: f64, right: f64) -> bool {
    !equals(left, right)
}

fn less_than(left: f64, right: f64) -> bool {
    left < right
}

fn greater_than(left: f64, right: f64) -> bool {
    left > right
}

fn less_than_equal(left: f64, right: f64) -> bool {
    left <= right
}

fn greater_than_equal(left: f64, right: f64) -> bool {
    left >= right
}

// Environment to store variables
struct Environment {
    variables: BTreeMap<String, f64>,
}

impl Environment {
    fn new() -> Self {
        Environment {
            variables: BTreeMap::new(),
        }
    }

    fn set(&mut self, name: String, value: f64) {
        self.variables.insert(name, value);
    }

    fn get(&self, name: &str) -> Option<f64> {
        self.variables.get(name).copied()
    }
}

lazy_static! {
    static ref GLOBAL_ENV: Mutex<Environment> = Mutex::new(Environment::new());
}
// Parser for expressions with precedence
struct Parser {
    tokens: Vec<Tokens>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Tokens>) -> Self {
        Parser {
            tokens,
            current: 0,
        }
    }

    fn parse(&mut self) -> f64 {
        self.expression()
    }

    fn expression(&mut self) -> f64 {
        // Handle variable assignment
        if let Tokens::Identifier(name) = self.peek(0).clone() {
            if let Tokens::Assign = self.peek(1) {
                let var_name = name;
                self.advance(); // consume identifier
                self.advance(); // consume =
                let value = self.expression();
                GLOBAL_ENV.lock().set(var_name, value);
                return value;
            }
        }
        
        self.comparison()
    }

    fn comparison(&mut self) -> f64 {
        let mut expr = self.term();

        while matches!(self.peek(0), 
            Tokens::Equal | Tokens::NotEqual | 
            Tokens::LessThan | Tokens::GreaterThan | 
            Tokens::LessThanEqual | Tokens::GreaterThanEqual) {
            
            let operator = self.advance().clone();
            let right = self.term();
            
            // For comparison operators, we return 1.0 for true and 0.0 for false
            expr = match operator {
                Tokens::Equal => if equals(expr, right) { 1.0 } else { 0.0 },
                Tokens::NotEqual => if not_equals(expr, right) { 1.0 } else { 0.0 },
                Tokens::LessThan => if less_than(expr, right) { 1.0 } else { 0.0 },
                Tokens::GreaterThan => if greater_than(expr, right) { 1.0 } else { 0.0 },
                Tokens::LessThanEqual => if less_than_equal(expr, right) { 1.0 } else { 0.0 },
                Tokens::GreaterThanEqual => if greater_than_equal(expr, right) { 1.0 } else { 0.0 },
                _ => unreachable!(),
            };
        }

        expr
    }

    fn term(&mut self) -> f64 {
        let mut expr = self.factor();

        while matches!(self.peek(0), Tokens::Plus | Tokens::Minus) {
            let operator = self.advance().clone();
            let right = self.factor();
            
            expr = match operator {
                Tokens::Plus => add(expr, right),
                Tokens::Minus => subtract(expr, right),
                _ => unreachable!(),
            };
        }

        expr
    }

    fn factor(&mut self) -> f64 {
        let mut expr = self.exponent();

        while matches!(self.peek(0), Tokens::Multiply | Tokens::Divide) {
            let operator = self.advance().clone();
            let right = self.exponent();
            
            expr = match operator {
                Tokens::Multiply => multiply(expr, right),
                Tokens::Divide => divide(expr, right),
                _ => unreachable!(),
            };
        }

        expr
    }

    fn exponent(&mut self) -> f64 {
        let mut expr = self.unary();

        while matches!(self.peek(0), Tokens::Power) {
            self.advance(); // consume ^
            let right = self.unary();
            expr = power(expr, right);
        }

        expr
    }

    fn unary(&mut self) -> f64 {
        if matches!(self.peek(0), Tokens::Minus | Tokens::Not) {
            let operator = self.advance().clone();
            let right = self.unary();
            
            return match operator {
                Tokens::Minus => -right,
                Tokens::Not => if right == 0.0 { 1.0 } else { 0.0 },
                _ => unreachable!(),
            };
        }

        self.primary()
    }

    fn primary(&mut self) -> f64 {
        let token = self.advance().clone();
        
        match token {
            Tokens::Number(value) => value,
            Tokens::Identifier(name) => {
                match GLOBAL_ENV.lock().get(&name) {
                    Some(value) => value,
                    None => {
                        println!("Undefined variable: {}", name);
                        0.0
                    }
                }
            },
            Tokens::LeftParen => {
                let expr = self.expression();
                if !matches!(self.advance(), Tokens::RightParen) {
                    println!("Expected closing parenthesis");
                }
                expr
            },
            Tokens::True => 1.0,
            Tokens::False => 0.0,
            _ => {
                println!("Unexpected token: {:?}", token);
                0.0
            }
        }
    }

    fn peek(&self, offset: usize) -> &Tokens {
        let position = self.current + offset;
        if position >= self.tokens.len() {
            &Tokens::EOF
        } else {
            &self.tokens[position]
        }
    }

    fn advance(&mut self) -> &Tokens {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(0), Tokens::EOF)
    }

    fn previous(&self) -> &Tokens {
        &self.tokens[self.current - 1]
    }
}

pub fn run_example() {
    let input = test();
    println!("Input:\n{}", input);
    
    // Reset the global environment
    *GLOBAL_ENV.lock() = Environment::new();
    
    // Split the input into lines
    let lines: Vec<&str> = input.lines().collect();
    
    for line in lines {
        // Skip empty lines and comments
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        
        println!("Evaluating: {}", trimmed);
        
        // Tokenize and parse each line
        let tokens = lexer(trimmed);
        
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        println!("Result: {}", result);
        
        println!("---");
    }
}

pub fn evaluate(input: &str) -> f64 {
    let tokens = lexer(input);
    let mut parser = Parser::new(tokens);
    parser.parse()
}