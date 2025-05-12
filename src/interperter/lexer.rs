extern crate alloc;
extern crate libm;

use crate::text_editor::express_editor::test;
use crate::{print, println};
use alloc::collections::BTreeMap;
use alloc::string::{self, String, ToString};
use alloc::vec::Vec;
use core::arch::asm;
use lazy_static::lazy_static;
use spin::Mutex;

#[derive(Debug, PartialEq, Clone)]
pub enum Tokens {
    Print,
    Println,
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
                    "print" => tokens.push(Tokens::Print),
                    "println" => tokens.push(Tokens::Println),
                    "++" => tokens.push(Tokens::Increment),
                    "--" => tokens.push(Tokens::Decrement),
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
        Parser { tokens, current: 0 }
    }

    fn parse(&mut self) -> f64 {
        self.statement()
    }

    fn expression(&mut self) -> f64 {
        // Handle variable assignment (only for existing variables)
        if let Tokens::Identifier(name) = self.peek(0).clone() {
            if let Tokens::Assign = self.peek(1) {
                // Check if the variable exists
                if GLOBAL_ENV.lock().get(&name).is_none() {
                    // Error will be handled in statement() method
                    // Just proceed with parsing
                    let var_name = name;
                    self.advance(); // consume identifier
                    self.advance(); // consume =
                    let value = self.expression();
                    return value;
                } else {
                    // Variable exists, proceed with assignment
                    let var_name = name;
                    self.advance(); // consume identifier
                    self.advance(); // consume =
                    let value = self.expression();
                    GLOBAL_ENV.lock().set(var_name, value);
                    return value;
                }
            }
        }

        self.comparison()
    }

    fn comparison(&mut self) -> f64 {
        let mut expr = self.term();

        while matches!(
            self.peek(0),
            Tokens::Equal
                | Tokens::NotEqual
                | Tokens::LessThan
                | Tokens::GreaterThan
                | Tokens::LessThanEqual
                | Tokens::GreaterThanEqual
        ) {
            let operator = self.advance().clone();
            let right = self.term();

            // For comparison operators, we return 1.0 for true and 0.0 for false
            expr = match operator {
                Tokens::Equal => {
                    if equals(expr, right) {
                        1.0
                    } else {
                        0.0
                    }
                }
                Tokens::NotEqual => {
                    if not_equals(expr, right) {
                        1.0
                    } else {
                        0.0
                    }
                }
                Tokens::LessThan => {
                    if less_than(expr, right) {
                        1.0
                    } else {
                        0.0
                    }
                }
                Tokens::GreaterThan => {
                    if greater_than(expr, right) {
                        1.0
                    } else {
                        0.0
                    }
                }
                Tokens::LessThanEqual => {
                    if less_than_equal(expr, right) {
                        1.0
                    } else {
                        0.0
                    }
                }
                Tokens::GreaterThanEqual => {
                    if greater_than_equal(expr, right) {
                        1.0
                    } else {
                        0.0
                    }
                }
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
                Tokens::Not => {
                    if right == 0.0 {
                        1.0
                    } else {
                        0.0
                    }
                }
                _ => unreachable!(),
            };
        }

        self.primary()
    }

    fn primary(&mut self) -> f64 {
        let token = self.advance().clone();

        match token {
            Tokens::Number(value) => value,
            Tokens::Identifier(name) => match GLOBAL_ENV.lock().get(&name) {
                Some(value) => value,
                None => {
                    println!("Undefined variable: {}", name);
                    0.0
                }
            },
            Tokens::LeftParen => {
                let expr = self.expression();
                if !matches!(self.advance(), Tokens::RightParen) {
                    println!("Expected closing parenthesis");
                }
                expr
            }
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

    fn statement(&mut self) -> f64 {
        if matches!(self.peek(0), Tokens::Print) {
            self.advance(); // consume 'print'

            // Check for opening parenthesis
            if !matches!(self.advance(), Tokens::LeftParen) {
                println!("Expected '(' after 'print'");
                return 0.0;
            }

            // Parse the expression or string to print
            let value = match self.peek(0) {
                Tokens::String(s) => {
                    let string_to_print = s.clone();
                    self.advance(); // consume the string
                    print!("{}", string_to_print);
                    0.0 // Return value doesn't matter for print
                }
                _ => {
                    // For expressions, evaluate and print the result
                    let result = self.expression();
                    print!("{}", result);
                    result
                }
            };

            // Check for closing parenthesis
            if !matches!(self.advance(), Tokens::RightParen) {
                println!("Expected ')' after print argument");
            }

            // Check for semicolon
            if !matches!(self.advance(), Tokens::Semicolon) {
                println!("Expected ';' after each statement/variable");
            }

            return value;
        }

        if matches!(self.peek(0), Tokens::Println) {
            self.advance(); // consume 'println'

            // Check for opening parenthesis
            if !matches!(self.advance(), Tokens::LeftParen) {
                println!("Expected '(' after 'println'");
                return 0.0;
            }

            // Parse the expression or string to print
            let value = match self.peek(0) {
                Tokens::String(s) => {
                    let string_to_print = s.clone();
                    self.advance(); // consume the string
                    println!("{}", string_to_print); // Actually print the string
                    0.0 // Return value doesn't matter for print
                }
                _ => {
                    // For expressions, evaluate and print the result
                    let result = self.expression();
                    println!("{}", result); // Print the result
                    result
                }
            };

            // Check for closing parenthesis
            if !matches!(self.advance(), Tokens::RightParen) {
                println!("Expected ')' after println argument");
            }

            // Check for semicolon
            if !matches!(self.advance(), Tokens::Semicolon) {
                println!("Expected ';' after each statement/variable");
            }

            return value;
        }

        // Variable declaration - require 'let' keyword
        if matches!(self.peek(0), Tokens::Let) {
            self.advance(); // consume 'let'

            // Expect an identifier
            let var_name = match self.peek(0) {
                Tokens::Identifier(name) => name.clone(),
                _ => {
                    println!("Expected identifier after 'let'");
                    return 0.0;
                }
            };
            self.advance(); // consume identifier

            // Expect an assignment
            if !matches!(self.peek(0), Tokens::Assign) {
                println!("Expected '=' after variable name");
                return 0.0;
            }
            self.advance(); // consume '='

            // Parse the expression
            let value = self.expression();

            // Store the variable
            GLOBAL_ENV.lock().set(var_name, value);

            // Expect a semicolon
            if !matches!(self.advance(), Tokens::Semicolon) {
                println!("Expected ';' after each statement/variable");
            }

            return value;
        }

        // If statement handling
        if matches!(self.peek(0), Tokens::If) {
            self.advance(); // consume 'if'

            // Parse condition
            let condition = self.expression();

            // Debug the condition value
            println!("If condition evaluated to: {}", condition);

            // Expect opening brace for then block
            if !matches!(self.advance(), Tokens::LeftBrace) {
                println!("Expected '{{' after if condition");
                return 0.0;
            }

            // Parse then block
            let mut then_result = 0.0;
            if condition != 0.0 {
                // Only execute the then block if condition is true
                println!("Executing 'then' block");
                then_result = self.block();
            } else {
                // Skip the then block
                println!("Skipping 'then' block");
                self.skip_block();
            }

            // Check for else
            if matches!(self.peek(0), Tokens::Else) {
                self.advance(); // consume 'else'

                // Expect opening brace for else block
                if !matches!(self.advance(), Tokens::LeftBrace) {
                    println!("Expected '{{' after else");
                    return 0.0;
                }

                // Parse else block
                if condition == 0.0 {
                    // Only execute the else block if condition is false
                    println!("Executing 'else' block");
                    return self.block();
                } else {
                    // Skip the else block
                    println!("Skipping 'else' block");
                    self.skip_block();
                    return then_result;
                }
            }

            return then_result;
        }

        // While loop handling
        if matches!(self.peek(0), Tokens::While) {
            self.advance(); // consume 'while'

            // Save the position of the condition
            let condition_pos = self.current;

            // Parse condition
            let mut condition = self.expression();

            // Expect opening brace for loop body
            if !matches!(self.advance(), Tokens::LeftBrace) {
                println!("Expected '{{' after while condition");
                return 0.0;
            }

            // Save the position of the loop body
            let body_pos = self.current;

            let mut last_value = 0.0;

            // Execute the loop as long as the condition is true
            while condition != 0.0 {
                // Execute the loop body
                last_value = self.block();

                // Go back to the condition
                self.current = condition_pos;

                // Re-evaluate the condition
                condition = self.expression();

                // Skip the opening brace
                self.advance();

                // Reset to the beginning of the loop body
                self.current = body_pos;
            }

            // Skip the loop body since the condition is now false
            self.skip_block();

            return last_value;
        }

        // Not a special statement, parse as normal expression
        // Check if this is a variable assignment (without 'let')
        if let Tokens::Identifier(name) = self.peek(0).clone() {
            if let Tokens::Assign = self.peek(1) {
                // This is a variable assignment without 'let'
                // Check if the variable already exists
                if GLOBAL_ENV.lock().get(&name).is_none() {
                    println!(
                        "Error: Variable '{}' must be declared with 'let' before assignment",
                        name
                    );
                    self.advance(); // consume identifier
                    self.advance(); // consume =
                    // Still evaluate the expression to avoid further parsing errors
                    let _ = self.expression();

                    // Check for semicolon
                    if !matches!(self.advance(), Tokens::Semicolon) {
                        println!("Expected ';' after each statement/variable");
                    }

                    return 0.0;
                }
                // If the variable exists, proceed with normal expression parsing
            }
        }

        let result = self.expression();

        // Consume and check for semicolon
        if !matches!(self.advance(), Tokens::Semicolon) {
            println!("Expected ';' after each statement/variable");
        }

        result
    }

    fn declaration(&mut self) -> f64 {
        if matches!(self.peek(0), Tokens::Fn) {
            self.advance(); // consume 'fn'

            // Expect a function name (identifier)
            let fn_name = match self.peek(0) {
                Tokens::Identifier(name) => name.clone(),
                _ => {
                    println!("Expected function name after 'fn'");
                    return 0.0;
                }
            };
            self.advance(); // consume function name

            // Expect opening parenthesis for parameters
            if !matches!(self.advance(), Tokens::LeftParen) {
                println!("Expected '(' after function name");
                return 0.0;
            }

            // Parse parameters (for now, just skip them)
            let mut params = Vec::new();
            if !matches!(self.peek(0), Tokens::RightParen) {
                loop {
                    // Expect parameter name
                    if let Tokens::Identifier(param_name) = self.peek(0).clone() {
                        params.push(param_name);
                        self.advance(); // consume parameter name

                        // Check for comma or closing parenthesis
                        if matches!(self.peek(0), Tokens::Comma) {
                            self.advance(); // consume comma
                        } else if matches!(self.peek(0), Tokens::RightParen) {
                            break;
                        } else {
                            println!("Expected ',' or ')' after parameter");
                            return 0.0;
                        }
                    } else {
                        println!("Expected parameter name");
                        return 0.0;
                    }
                }

                self.advance();

                if !matches!(self.advance(), Tokens::LeftBrace) {
                    println!("Expected '{{' after function parameters or ()");
                }

                self.block();

                return 0.0
            }
        }

        self.statement()
    }

    // Helper methods for block handling
    fn block(&mut self) -> f64 {
        let mut last_value = 0.0;
        let mut brace_count = 1; // We've already consumed the opening brace

        while brace_count > 0 && !self.is_at_end() {
            // Check if we've reached a closing brace
            if matches!(self.peek(0), Tokens::RightBrace) {
                brace_count -= 1;
                if brace_count == 0 {
                    self.advance(); // consume the closing brace
                    break;
                }
                self.advance(); // consume the closing brace of a nested block
                continue;
            }

            // Check for opening braces (for nested blocks)
            if matches!(self.peek(0), Tokens::LeftBrace) {
                brace_count += 1;
                self.advance(); // consume the opening brace
                continue;
            }

            // Process a statement
            last_value = self.statement();
        }

        last_value
    }

    fn skip_block(&mut self) {
        let mut brace_count = 1; // We've already consumed the opening brace

        while brace_count > 0 && !self.is_at_end() {
            self.advance(); // Skip this token

            if matches!(self.previous(), Tokens::LeftBrace) {
                brace_count += 1;
            } else if matches!(self.previous(), Tokens::RightBrace) {
                brace_count -= 1;
                if brace_count == 0 {
                    break;
                }
            }
        }
    }
}

pub fn run_example() {
    let input = test();
    println!("Input:\n{}", input);

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
        println!("Evaluating: {}", block);

        // Tokenize and parse the block
        let tokens = lexer(&block);

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
