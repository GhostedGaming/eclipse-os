extern crate alloc;

use crate::{print, println};
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::{serial_print,serial_println};

// ================ TOKENS ================
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    Keyword(Keyword),
    // Values
    Identifier(String),
    Number(f64),
    StringLiteral(String),
    // Operators
    Operator(Operator),
    // Shell specific
    Command(String),
    Argument(String),
    Pipe,
    Redirect(RedirectType),
    // Structure
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    Comma,
    EOF,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    Let,
    If,
    Else,
    While,
    For,
    Echo,
    Cd,
    Pwd,
    Exit,
    Export,
    True,
    False,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    // Arithmetic
    Plus,
    Minus,
    Multiply,
    Divide,
    // Assignment
    Assign,
    // Comparison
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    // Logical
    And,
    Or,
    Not,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RedirectType {
    Out,    // >
    Append, // >>
    In,     // <
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Keyword(kw) => write!(f, "{:?}", kw),
            Token::Identifier(name) => write!(f, "ID({})", name),
            Token::Number(num) => write!(f, "NUM({})", num),
            Token::StringLiteral(s) => write!(f, "STR(\"{}\")", s),
            Token::Operator(op) => write!(f, "{:?}", op),
            Token::Command(cmd) => write!(f, "CMD({})", cmd),
            Token::Argument(arg) => write!(f, "ARG({})", arg),
            Token::Pipe => write!(f, "|"),
            Token::Redirect(rtype) => match rtype {
                RedirectType::Out => write!(f, ">"),
                RedirectType::Append => write!(f, ">>"),
                RedirectType::In => write!(f, "<"),
            },
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::Semicolon => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::EOF => write!(f, "EOF"),
        }
    }
}

// ================ LEXER ================
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    read_position: usize,
    ch: Option<char>,
    mode: LexerMode,
}

#[derive(PartialEq)]
enum LexerMode {
    Normal,
    Command,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            read_position: 0,
            ch: None,
            mode: LexerMode::Normal,
        };
        lexer.read_char();
        lexer
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = None;
        } else {
            self.ch = self.input.chars().nth(self.read_position);
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    fn peek_char(&self) -> Option<char> {
        if self.read_position >= self.input.len() {
            None
        } else {
            self.input.chars().nth(self.read_position)
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.ch {
            if ch.is_whitespace() {
                self.read_char();
            } else {
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        let position = self.position;
        while let Some(ch) = self.ch {
            if ch.is_alphanumeric() || ch == '_' {
                self.read_char();
            } else {
                break;
            }
        }
        self.input[position..self.position].to_string()
    }

    fn read_command_arg(&mut self) -> String {
        let position = self.position;
        while let Some(ch) = self.ch {
            // Command arguments can contain most characters except whitespace and special shell chars
            if !ch.is_whitespace() && ch != '|' && ch != '>' && ch != '<' && ch != ';' {
                self.read_char();
            } else {
                break;
            }
        }
        self.input[position..self.position].to_string()
    }

    fn read_number(&mut self) -> String {
        let position = self.position;
        while let Some(ch) = self.ch {
            if ch.is_digit(10) || ch == '.' {
                self.read_char();
            } else {
                break;
            }
        }
        self.input[position..self.position].to_string()
    }

    fn read_string(&mut self) -> String {
        self.read_char(); // Skip the opening quote
        let position = self.position;

        while let Some(ch) = self.ch {
            if ch == '"' || ch == '\0' {
                break;
            }
            self.read_char();
        }

        let str = self.input[position..self.position].to_string();
        self.read_char(); // Skip the closing quote
        str
    }

    fn read_env_var(&mut self) -> String {
        self.read_char(); // Skip the $ symbol
        self.read_identifier()
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.ch {
            None => Token::EOF,
            Some(ch) => {
                // If in command mode and not at start of line, treat non-special chars as arguments
                if self.mode == LexerMode::Command
                    && !ch.is_whitespace()
                    && ch != '|'
                    && ch != '>'
                    && ch != '<'
                    && ch != ';'
                    && ch != '('
                    && ch != ')'
                {
                    let arg = self.read_command_arg();
                    return Token::Argument(arg);
                }

                match ch {
                    // Single character tokens
                    '=' => {
                        if self.peek_char() == Some('=') {
                            self.read_char();
                            Token::Operator(Operator::Equal)
                        } else {
                            Token::Operator(Operator::Assign)
                        }
                    }
                    '+' => Token::Operator(Operator::Plus),
                    '-' => Token::Operator(Operator::Minus),
                    '*' => Token::Operator(Operator::Multiply),
                    '/' => {
                        if self.peek_char() == Some('/') {
                            // Skip comments
                            while self.ch.is_some() && self.ch != Some('\n') {
                                self.read_char();
                            }
                            return self.next_token();
                        } else {
                            Token::Operator(Operator::Divide)
                        }
                    }
                    '!' => {
                        if self.peek_char() == Some('=') {
                            self.read_char();
                            Token::Operator(Operator::NotEqual)
                        } else {
                            Token::Operator(Operator::Not)
                        }
                    }
                    '<' => Token::Redirect(RedirectType::In),
                    '>' => {
                        if self.peek_char() == Some('>') {
                            self.read_char();
                            Token::Redirect(RedirectType::Append)
                        } else {
                            Token::Redirect(RedirectType::Out)
                        }
                    }
                    '&' => {
                        if self.peek_char() == Some('&') {
                            self.read_char();
                            Token::Operator(Operator::And)
                        } else {
                            // Background process operator - for future implementation
                            self.read_char();
                            return self.next_token();
                        }
                    }
                    '|' => {
                        if self.peek_char() == Some('|') {
                            self.read_char();
                            Token::Operator(Operator::Or)
                        } else {
                            Token::Pipe
                        }
                    }
                    '(' => Token::LeftParen,
                    ')' => Token::RightParen,
                    '{' => Token::LeftBrace,
                    '}' => Token::RightBrace,
                    ';' => {
                        self.mode = LexerMode::Normal; // Reset to normal mode after command
                        Token::Semicolon
                    }
                    ',' => Token::Comma,
                    '"' => Token::StringLiteral(self.read_string()),
                    '$' => {
                        // Handle environment variable
                        let var_name = self.read_env_var();
                        // Return as identifier, will be resolved during evaluation
                        Token::Identifier(var_name)
                    }
                    _ => {
                        if ch.is_alphabetic() || ch == '_' {
                            let ident = self.read_identifier();

                            // If this is the first token in a statement, it might be a command
                            if self.mode == LexerMode::Normal {
                                self.mode = LexerMode::Command; // Switch to command mode

                                match ident.as_str() {
                                    "let" => Token::Keyword(Keyword::Let),
                                    "if" => Token::Keyword(Keyword::If),
                                    "else" => Token::Keyword(Keyword::Else),
                                    "while" => Token::Keyword(Keyword::While),
                                    "for" => Token::Keyword(Keyword::For),
                                    "echo" => Token::Keyword(Keyword::Echo),
                                    "cd" => Token::Keyword(Keyword::Cd),
                                    "pwd" => Token::Keyword(Keyword::Pwd),
                                    "exit" => Token::Keyword(Keyword::Exit),
                                    "export" => Token::Keyword(Keyword::Export),
                                    "true" => Token::Keyword(Keyword::True),
                                    "false" => Token::Keyword(Keyword::False),
                                    _ => Token::Command(ident),
                                }
                            } else {
                                Token::Argument(ident)
                            }
                        } else if ch.is_digit(10) {
                            let num_str = self.read_number();
                            match num_str.parse::<f64>() {
                                Ok(num) => Token::Number(num),
                                Err(_) => {
                                    println!("Invalid number: {}", num_str);
                                    self.read_char();
                                    return self.next_token();
                                }
                            }
                        } else {
                            println!("Unexpected character: {}", ch);
                            self.read_char();
                            return self.next_token();
                        }
                    }
                }
            }
        };

        self.read_char();
        token
    }
}

// ================ VALUE ================
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    CommandOutput(String),
    ExitCode(i32),
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::CommandOutput(s) => write!(f, "{}", s),
            Value::ExitCode(code) => write!(f, "Exit code: {}", code),
            Value::Null => write!(f, ""),
        }
    }
}

// ================ ENVIRONMENT ================
#[derive(Debug, Clone)]
pub struct Environment {
    variables: BTreeMap<String, Value>,
    current_dir: String,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            variables: BTreeMap::new(),
            current_dir: "/".to_string(), // Default root directory
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        // Check built-in variables first
        match name {
            "PWD" => Some(Value::String(self.current_dir.clone())),
            "HOME" => Some(Value::String("/home/user".to_string())), // Default home
            _ => self.variables.get(name).cloned(),
        }
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_current_dir(&self) -> String {
        self.current_dir.clone()
    }

    pub fn set_current_dir(&mut self, dir: String) -> Result<(), String> {
        // In a real OS, we would validate the directory here
        // For our shell simulator, we'll just check some basic rules

        if dir.starts_with('/') {
            // Absolute path
            self.current_dir = dir;
        } else if dir == ".." {
            // Go up one directory
            let parts: Vec<&str> = self.current_dir.split('/').collect();
            if parts.len() > 1 {
                self.current_dir = parts[..parts.len() - 1].join("/");
                if self.current_dir.is_empty() {
                    self.current_dir = "/".to_string();
                }
            }
        } else if !dir.contains("..") {
            // Relative path without ..
            if self.current_dir.ends_with('/') {
                self.current_dir = format!("{}{}", self.current_dir, dir);
            } else {
                self.current_dir = format!("{}/{}", self.current_dir, dir);
            }
        } else {
            return Err(format!("Invalid directory: {}", dir));
        }

        Ok(())
    }
}

lazy_static! {
    static ref GLOBAL_ENV: Mutex<Environment> = Mutex::new(Environment::new());
}

// ================ COMMAND EXECUTION ================
#[derive(Debug)]
pub struct Command {
    program: String,
    args: Vec<String>,
}

impl Command {
    pub fn new(program: String) -> Self {
        Command {
            program,
            args: Vec::new(),
        }
    }

    pub fn add_arg(&mut self, arg: String) {
        self.args.push(arg);
    }

    pub fn execute(&self) -> Result<Value, String> {
        // Since we're in a no_std environment, we'll simulate command execution
        match self.program.as_str() {
            "cd" => {
                let dir = if self.args.is_empty() {
                    match GLOBAL_ENV.lock().get("HOME") {
                        Some(Value::String(home)) => home,
                        _ => return Err("HOME environment variable not set".to_string()),
                    }
                } else {
                    self.args[0].clone()
                };

                match GLOBAL_ENV.lock().set_current_dir(dir.clone()) {
                    Ok(_) => Ok(Value::Null),
                    Err(e) => Err(format!("cd: {}: {}", dir, e)),
                }
            }
            "pwd" => {
                let current_dir = GLOBAL_ENV.lock().get_current_dir();
                println!("{}", current_dir);
                Ok(Value::String(current_dir))
            }
            "echo" => {
                let output = self.args.join(" ");
                println!("{}", output);
                Ok(Value::String(output))
            }
            "ls" => {
                // Simulate listing files in current directory
                let files = ["file1.txt", "file2.txt", "directory1", "directory2"];

                let output = files.join("\n");
                println!("{}", output);
                Ok(Value::CommandOutput(output))
            }
            "cat" => {
                if self.args.is_empty() {
                    return Err("cat: missing file operand".to_string());
                }

                // Simulate file content
                let filename = &self.args[0];
                let content = format!("Content of file {}", filename);
                println!("{}", content);
                Ok(Value::CommandOutput(content))
            }
            "exit" => {
                let code = if self.args.is_empty() {
                    0
                } else {
                    match self.args[0].parse::<i32>() {
                        Ok(code) => code,
                        Err(_) => 1,
                    }
                };

                // In a real implementation, we would exit the process
                // In our simulator, we'll just return the exit code
                Ok(Value::ExitCode(code))
            }
            _ => {
                // Command not found
                Err(format!("Command not found: {}", self.program))
            }
        }
    }
}

// ================ PARSER ================
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Value, String> {
        if self.is_at_end() {
            return Ok(Value::Null);
        }

        match self.peek() {
            Token::Keyword(_) => self.parse_keyword(),
            Token::Command(_) => self.parse_command(),
            _ => {
                // For other tokens, try to parse an expression
                self.expression()
            }
        }
    }

    fn parse_keyword(&mut self) -> Result<Value, String> {
        match self.peek() {
            Token::Keyword(Keyword::Let) => self.parse_let(),
            Token::Keyword(Keyword::If) => self.parse_if(),
            Token::Keyword(Keyword::While) => self.parse_while(),
            Token::Keyword(Keyword::For) => self.parse_for(),
            Token::Keyword(Keyword::Echo) => self.parse_echo(),
            Token::Keyword(Keyword::Cd) => self.parse_cd(),
            Token::Keyword(Keyword::Pwd) => self.parse_pwd(),
            Token::Keyword(Keyword::Exit) => self.parse_exit(),
            Token::Keyword(Keyword::Export) => self.parse_export(),
            _ => Err(format!("Unexpected keyword: {:?}", self.peek())),
        }
    }

    fn parse_let(&mut self) -> Result<Value, String> {
        self.advance(); // Consume 'let'

        // Expect identifier
        if let Token::Identifier(name) = self.advance() {
            // Expect assignment operator
            if let Token::Operator(Operator::Assign) = self.advance() {
                // Parse the expression after =
                let value = self.expression()?;

                // Create variable in environment
                GLOBAL_ENV.lock().set(name, value.clone());

                if self.match_token(Token::Semicolon) {
                    Ok(value)
                } else {
                    Err("Expected semicolon after let statement".to_string())
                }
            } else {
                Err("Expected '=' after variable name".to_string())
            }
        } else {
            Err("Expected identifier after 'let'".to_string())
        }
    }

    fn parse_if(&mut self) -> Result<Value, String> {
        self.advance(); // Consume 'if'

        // Expect condition in parentheses
        if !self.match_token(Token::LeftParen) {
            return Err("Expected '(' after 'if'".to_string());
        }

        let condition = self.expression()?;

        if !self.match_token(Token::RightParen) {
            return Err("Expected ')' after condition".to_string());
        }

        // Expect brace for then block
        if !self.match_token(Token::LeftBrace) {
            return Err("Expected '{' before if block".to_string());
        }

        // Parse then block
        let then_value = if self.is_true(&condition) {
            self.block()?
        } else {
            self.skip_block()?;
            Value::Null
        };

        // Check for else
        if self.match_token(Token::Keyword(Keyword::Else)) {
            // Expect brace for else block
            if !self.match_token(Token::LeftBrace) {
                return Err("Expected '{' before else block".to_string());
            }

            // Parse else block
            if self.is_true(&condition) {
                self.skip_block()?;
                Ok(then_value)
            } else {
                self.block()
            }
        } else {
            Ok(then_value)
        }
    }

    fn parse_while(&mut self) -> Result<Value, String> {
        self.advance(); // Consume 'while'

        // Expect condition in parentheses
        if !self.match_token(Token::LeftParen) {
            return Err("Expected '(' after 'while'".to_string());
        }

        // Save position of condition
        let condition_pos = self.current;

        let mut condition = self.expression()?;

        if !self.match_token(Token::RightParen) {
            return Err("Expected ')' after condition".to_string());
        }

        // Expect brace for loop body
        if !self.match_token(Token::LeftBrace) {
            return Err("Expected '{' before while block".to_string());
        }

        // Save position of body
        let body_pos = self.current;

        let mut last_value = Value::Null;

        // Execute loop while condition is true
        while self.is_true(&condition) {
            last_value = self.block()?;

            // Go back to condition
            self.current = condition_pos;
            condition = self.expression()?;

            // Skip the closing paren again
            self.advance();

            // Skip the opening brace again
            self.advance();

            // Reset to body position
            self.current = body_pos;
        }

        // Skip the body since condition is now false
        self.skip_block()?;

        Ok(last_value)
    }

    fn parse_for(&mut self) -> Result<Value, String> {
        self.advance(); // Consume 'for'

        // Expect variable name
        let var_name = if let Token::Identifier(name) = self.advance() {
            name
        } else {
            return Err("Expected variable name after 'for'".to_string());
        };

        // Expect 'in' keyword (could extend Token enum for this)
        if let Token::Identifier(keyword) = self.advance() {
            if keyword != "in" {
                return Err("Expected 'in' after variable name in for loop".to_string());
            }
        } else {
            return Err("Expected 'in' after variable name in for loop".to_string());
        }

        // Expect an iterable expression
        let iterable = self.expression()?;

        // Convert iterable to a list of values
        let values = match iterable {
            Value::String(s) => {
                // Iterate over characters
                s.chars()
                    .map(|c| Value::String(c.to_string()))
                    .collect::<Vec<_>>()
            }
            Value::CommandOutput(output) => {
                // Split by newlines
                output
                    .lines()
                    .map(|line| Value::String(line.to_string()))
                    .collect::<Vec<_>>()
            }
            _ => return Err("Expected iterable value after 'in'".to_string()),
        };

        // Expect brace for loop body
        if !self.match_token(Token::LeftBrace) {
            return Err("Expected '{' before for loop body".to_string());
        }

        // Save position of body
        let body_pos = self.current;

        let mut last_value = Value::Null;

        // Iterate over values
        for value in values {
            // Set variable to current value
            GLOBAL_ENV.lock().set(var_name.clone(), value);

            // Execute loop body
            last_value = self.block()?;

            // Reset to body position
            self.current = body_pos;
        }

        // Skip the body after loop is done
        self.skip_block()?;

        Ok(last_value)
    }

    fn parse_echo(&mut self) -> Result<Value, String> {
        self.advance(); // Consume 'echo'

        // Collect all arguments until semicolon
        let mut args = Vec::new();

        while !self.check(Token::Semicolon) && !self.is_at_end() {
            if let Token::Argument(arg) = self.advance() {
                args.push(arg);
            } else {
                // If not an argument, try to evaluate as expression
                let arg = self.expression()?;
                args.push(arg.to_string());
            }
        }

        // Consume semicolon
        if !self.match_token(Token::Semicolon) {
            return Err("Expected ';' after echo command".to_string());
        }

        // Join arguments with spaces and print
        let output = args.join(" ");
        println!("{}", output);

        Ok(Value::String(output))
    }

    fn parse_cd(&mut self) -> Result<Value, String> {
        self.advance(); // Consume 'cd'

        // Get directory argument
        let dir = if let Token::Argument(dir) = self.advance() {
            dir
        } else if self.check(Token::Semicolon) {
            // No argument, use HOME
            match GLOBAL_ENV.lock().get("HOME") {
                Some(Value::String(home)) => home,
                _ => return Err("HOME environment variable not set".to_string()),
            }
        } else {
            return Err("Expected directory argument or semicolon after 'cd'".to_string());
        };

        // Consume semicolon
        if !self.match_token(Token::Semicolon) {
            return Err("Expected ';' after cd command".to_string());
        }

        // Change directory
        match GLOBAL_ENV.lock().set_current_dir(dir.clone()) {
            Ok(_) => Ok(Value::Null),
            Err(e) => Err(format!("cd: {}: {}", dir, e)),
        }
    }

    fn parse_pwd(&mut self) -> Result<Value, String> {
        self.advance(); // Consume 'pwd'

        // Consume semicolon
        if !self.match_token(Token::Semicolon) {
            return Err("Expected ';' after pwd command".to_string());
        }

        // Get current directory
        let path_str = GLOBAL_ENV.lock().get_current_dir();
        println!("{}", path_str);
        Ok(Value::String(path_str))
    }

    fn parse_exit(&mut self) -> Result<Value, String> {
        self.advance(); // Consume 'exit'

        // Get exit code if provided
        let code = if let Token::Argument(code_str) = self.peek() {
            let code_str = code_str.clone();
            self.advance(); // Consume argument
            match code_str.parse::<i32>() {
                Ok(code) => code,
                Err(_) => 1, // Invalid code defaults to 1
            }
        } else {
            0 // Default exit code is 0
        };

        // Consume semicolon
        if !self.match_token(Token::Semicolon) {
            return Err("Expected ';' after exit command".to_string());
        }

        // Return exit code
        Ok(Value::ExitCode(code))
    }

    fn parse_export(&mut self) -> Result<Value, String> {
        self.advance(); // Consume 'export'

        // Get variable name
        let var_name = if let Token::Argument(name) = self.advance() {
            name
        } else {
            return Err("Expected variable name after 'export'".to_string());
        };

        // Consume semicolon
        if !self.match_token(Token::Semicolon) {
            return Err("Expected ';' after export command".to_string());
        }

        // Check if variable exists
        if GLOBAL_ENV.lock().get(&var_name).is_none() {
            return Err(format!("Variable not found: {}", var_name));
        }

        // In a no_std environment, we can't actually export to the system environment
        // Just pretend it worked
        Ok(Value::Null)
    }

    fn parse_command(&mut self) -> Result<Value, String> {
        // Extract command name
        let cmd_name = if let Token::Command(name) = self.advance() {
            name
        } else {
            return Err("Expected command name".to_string());
        };

        let mut command = Command::new(cmd_name);

        // Parse arguments
        while !self.check(Token::Semicolon) && !self.check(Token::Pipe) && !self.is_at_end() {
            if let Token::Argument(arg) = self.advance() {
                // Expand variables in arguments
                let expanded_arg = self.expand_variables(&arg);
                command.add_arg(expanded_arg);
            } else {
                // If not an argument, this is an error in command syntax
                return Err(format!(
                    "Unexpected token in command: {:?}",
                    self.previous()
                ));
            }
        }

        // We don't handle pipes in the no_std version to keep it simple

        // Consume semicolon
        if !self.match_token(Token::Semicolon) && !self.is_at_end() {
            return Err("Expected ';' after command".to_string());
        }

        // Execute the command
        command.execute()
    }

    fn expand_variables(&self, arg: &str) -> String {
        let mut result = String::new();
        let mut i = 0;

        while i < arg.len() {
            if arg[i..].starts_with('$') && i + 1 < arg.len() {
                // Find the end of the variable name
                let mut j = i + 1;
                while j < arg.len()
                    && (arg.chars().nth(j).unwrap().is_alphanumeric()
                        || arg.chars().nth(j).unwrap() == '_')
                {
                    j += 1;
                }

                // Extract variable name
                let var_name = &arg[i + 1..j];

                // Look up variable value
                if let Some(value) = GLOBAL_ENV.lock().get(var_name) {
                    result.push_str(&value.to_string());
                } else {
                    // Variable not found, leave as is
                    result.push('$');
                    result.push_str(var_name);
                }

                i = j;
            } else {
                // Regular character
                result.push(arg.chars().nth(i).unwrap());
                i += 1;
            }
        }

        result
    }

    fn block(&mut self) -> Result<Value, String> {
        let mut last_value = Value::Null;
        let mut brace_count = 1;

        while brace_count > 0 && !self.is_at_end() {
            match self.peek() {
                Token::LeftBrace => {
                    brace_count += 1;
                    self.advance();
                }
                Token::RightBrace => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        self.advance(); // Consume closing brace
                        break;
                    }
                    self.advance();
                }
                _ => {
                    // Parse statement
                    last_value = self.parse()?;
                }
            }
        }

        Ok(last_value)
    }

    fn skip_block(&mut self) -> Result<(), String> {
        let mut brace_count = 1;

        while brace_count > 0 && !self.is_at_end() {
            match self.advance() {
                Token::LeftBrace => brace_count += 1,
                Token::RightBrace => brace_count -= 1,
                _ => {}
            }
        }

        if brace_count > 0 {
            Err("Unexpected end of input while skipping block".to_string())
        } else {
            Ok(())
        }
    }

    fn expression(&mut self) -> Result<Value, String> {
        self.logical_or()
    }

    fn logical_or(&mut self) -> Result<Value, String> {
        let mut expr = self.logical_and()?;

        while self.match_token(Token::Operator(Operator::Or)) {
            let right = self.logical_and()?;

            // Short-circuit evaluation
            if self.is_true(&expr) {
                // Left is true, result is true regardless of right
                expr = Value::Boolean(true);
            } else if self.is_true(&right) {
                // Left is false, right is true, result is true
                expr = Value::Boolean(true);
            } else {
                // Both are false
                expr = Value::Boolean(false);
            }
        }

        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Value, String> {
        let mut expr = self.equality()?;

        while self.match_token(Token::Operator(Operator::And)) {
            let right = self.equality()?;

            // Short-circuit evaluation
            if !self.is_true(&expr) {
                // Left is false, result is false regardless of right
                expr = Value::Boolean(false);
            } else if !self.is_true(&right) {
                // Left is true, right is false, result is false
                expr = Value::Boolean(false);
            } else {
                // Both are true
                expr = Value::Boolean(true);
            }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Value, String> {
        let mut expr = self.comparison()?;

        while self.match_token(Token::Operator(Operator::Equal))
            || self.match_token(Token::Operator(Operator::NotEqual))
        {
            let op = self.previous();
            let right = self.comparison()?;

            expr = match (expr, right) {
                (Value::Number(a), Value::Number(b)) => {
                    if matches!(op, Token::Operator(Operator::Equal)) {
                        Value::Boolean((a - b).abs() < core::f64::EPSILON)
                    } else {
                        Value::Boolean((a - b).abs() >= core::f64::EPSILON)
                    }
                }
                (Value::String(a), Value::String(b)) => {
                    if matches!(op, Token::Operator(Operator::Equal)) {
                        Value::Boolean(a == b)
                    } else {
                        Value::Boolean(a != b)
                    }
                }
                (Value::Boolean(a), Value::Boolean(b)) => {
                    if matches!(op, Token::Operator(Operator::Equal)) {
                        Value::Boolean(a == b)
                    } else {
                        Value::Boolean(a != b)
                    }
                }
                _ => {
                    // Different types are never equal
                    if matches!(op, Token::Operator(Operator::Equal)) {
                        Value::Boolean(false)
                    } else {
                        Value::Boolean(true)
                    }
                }
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Value, String> {
        let mut expr = self.term()?;

        while self.match_token(Token::Operator(Operator::LessThan))
            || self.match_token(Token::Operator(Operator::GreaterThan))
            || self.match_token(Token::Operator(Operator::LessThanEqual))
            || self.match_token(Token::Operator(Operator::GreaterThanEqual))
        {
            let op = self.previous();
            let right = self.term()?;

            expr = match (expr, right) {
                (Value::Number(a), Value::Number(b)) => match op {
                    Token::Operator(Operator::LessThan) => Value::Boolean(a < b),
                    Token::Operator(Operator::GreaterThan) => Value::Boolean(a > b),
                    Token::Operator(Operator::LessThanEqual) => Value::Boolean(a <= b),
                    Token::Operator(Operator::GreaterThanEqual) => Value::Boolean(a >= b),
                    _ => unreachable!(),
                },
                (Value::String(a), Value::String(b)) => match op {
                    Token::Operator(Operator::LessThan) => Value::Boolean(a < b),
                    Token::Operator(Operator::GreaterThan) => Value::Boolean(a > b),
                    Token::Operator(Operator::LessThanEqual) => Value::Boolean(a <= b),
                    Token::Operator(Operator::GreaterThanEqual) => Value::Boolean(a >= b),
                    _ => unreachable!(),
                },
                _ => return Err("Cannot compare values of different types".to_string()),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Value, String> {
        let mut expr = self.factor()?;

        while self.match_token(Token::Operator(Operator::Plus))
            || self.match_token(Token::Operator(Operator::Minus))
        {
            let op = self.previous();
            let right = self.factor()?;

            expr = match (expr, right) {
                (Value::Number(a), Value::Number(b)) => {
                    if matches!(op, Token::Operator(Operator::Plus)) {
                        Value::Number(a + b)
                    } else {
                        Value::Number(a - b)
                    }
                }
                (Value::String(a), Value::String(b))
                    if matches!(op, Token::Operator(Operator::Plus)) =>
                {
                    Value::String(a + &b)
                }
                _ => return Err("Invalid operands for operator".to_string()),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Value, String> {
        let mut expr = self.unary()?;

        while self.match_token(Token::Operator(Operator::Multiply))
            || self.match_token(Token::Operator(Operator::Divide))
        {
            let op = self.previous();
            let right = self.unary()?;

            expr = match (expr, right) {
                (Value::Number(a), Value::Number(b)) => {
                    if matches!(op, Token::Operator(Operator::Multiply)) {
                        Value::Number(a * b)
                    } else {
                        if b == 0.0 {
                            return Err("Division by zero".to_string());
                        }
                        Value::Number(a / b)
                    }
                }
                _ => return Err("Invalid operands for operator".to_string()),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Value, String> {
        if self.match_token(Token::Operator(Operator::Minus))
            || self.match_token(Token::Operator(Operator::Not))
        {
            let op = self.previous();
            let right = self.unary()?;

            return match right {
                Value::Number(n) if matches!(op, Token::Operator(Operator::Minus)) => {
                    Ok(Value::Number(-n))
                }
                Value::Boolean(b) if matches!(op, Token::Operator(Operator::Not)) => {
                    Ok(Value::Boolean(!b))
                }
                _ => Err("Invalid operand for unary operator".to_string()),
            };
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Value, String> {
        if self.match_token(Token::Number(0.0)) {
            if let Token::Number(n) = self.previous() {
                return Ok(Value::Number(n));
            }
        }

        if self.match_token(Token::StringLiteral(String::new())) {
            if let Token::StringLiteral(s) = self.previous() {
                return Ok(Value::String(s));
            }
        }

        if self.match_token(Token::Keyword(Keyword::True)) {
            return Ok(Value::Boolean(true));
        }

        if self.match_token(Token::Keyword(Keyword::False)) {
            return Ok(Value::Boolean(false));
        }

        if self.match_token(Token::Identifier(String::new())) {
            if let Token::Identifier(name) = self.previous() {
                if let Some(value) = GLOBAL_ENV.lock().get(&name) {
                    return Ok(value);
                } else {
                    return Err(format!("Undefined variable: '{}'", name));
                }
            }
        }

        if self.match_token(Token::LeftParen) {
            let expr = self.expression()?;

            if !self.match_token(Token::RightParen) {
                return Err("Expected ')' after expression".to_string());
            }

            return Ok(expr);
        }

        Err(format!("Unexpected token: {:?}", self.peek()))
    }

    // Helper methods
    fn is_true(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::CommandOutput(s) => !s.is_empty(),
            Value::ExitCode(code) => *code == 0,
            Value::Null => false,
        }
    }

    fn match_token(&mut self, token: Token) -> bool {
        match (&token, self.peek()) {
            (Token::Number(_), Token::Number(_)) => {
                self.advance();
                true
            }
            (Token::StringLiteral(_), Token::StringLiteral(_)) => {
                self.advance();
                true
            }
            (Token::Identifier(_), Token::Identifier(_)) => {
                self.advance();
                true
            }
            _ => {
                if *self.peek() == token {
                    self.advance();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn check(&self, token: Token) -> bool {
        match (&token, self.peek()) {
            (Token::Number(_), Token::Number(_)) => true,
            (Token::StringLiteral(_), Token::StringLiteral(_)) => true,
            (Token::Identifier(_), Token::Identifier(_)) => true,
            _ => *self.peek() == token,
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        *self.peek() == Token::EOF
    }

    fn peek(&self) -> &Token {
        if self.current >= self.tokens.len() {
            &Token::EOF
        } else {
            &self.tokens[self.current]
        }
    }

    fn previous(&self) -> Token {
        if self.current == 0 {
            Token::EOF
        } else {
            self.tokens[self.current - 1].clone()
        }
    }
}

// ================ INTERPRETER ================
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next_token();
        tokens.push(token.clone());

        if matches!(token, Token::EOF) {
            break;
        }
    }

    tokens
}

pub fn parse_and_execute(input: &str) -> Result<Value, String> {
    let tokens = tokenize(input);

    let mut parser = Parser::new(tokens);
    parser.parse()
}

// Shell integration
pub fn run_shell() {
    println!("Simple Shell Interpreter v1.0 (no_std)");
    println!("Type 'exit' to quit");

    loop {
        // Print prompt with current directory
        print!("{}> ", GLOBAL_ENV.lock().get_current_dir());

        // In a real no_std environment, we would have a custom input mechanism
        // For simulation purposes, we'll just create some example commands

        let input = "echo Hello, no_std world!";
        println!("{}", input);

        match parse_and_execute(input) {
            Ok(value) => {
                // Only show non-null values that aren't already printed
                if !matches!(value, Value::Null) && !matches!(value, Value::CommandOutput(_)) {
                    println!("{}", value);
                }
            }
            Err(error) => println!("Error: {}", error),
        }

        // Simulate the exit command
        break;
    }

    println!("Goodbye!");
}

// Main function to demonstrate usage
pub fn run_example() {
    println!("Running shell example...");

    // Run some commands to demonstrate the shell
    let commands = [
        "let x = 10;",
        "let y = 20;",
        "let sum = x + y;",
        "echo Sum is $sum;",
        "if (x < y) { echo x is less than y; } else { echo x is not less than y; }",
        "let i = 0;",
        "while (i < 3) { echo i=$i; let i = i + 1; }",
        "pwd;",
        "cd /usr;",
        "pwd;",
        "ls;",
        "exit;",
    ];

    for cmd in &commands {
        println!("\n> {}", cmd);
        match parse_and_execute(cmd) {
            Ok(value) => {
                if !matches!(value, Value::Null)
                    && !matches!(value, Value::CommandOutput(_))
                    && !matches!(value, Value::ExitCode(_))
                {
                    println!("=> {}", value);
                }
            }
            Err(error) => println!("Error: {}", error),
        }
    }
}
