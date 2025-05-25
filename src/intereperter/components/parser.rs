use core::arch::asm;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::{print, println};
use super::Environment;
use super::GLOBAL_ENV;
use super::Tokens;
use super::add;
use super::divide;
use super::equals;
use super::greater_than;
use super::greater_than_equal;
use super::less_than;
use super::less_than_equal;
use super::multiply;
use super::not_equals;
use super::power;
use super::subtract;
use super::FUNCTION_REGISTRY;
use super::Function;

pub struct Parser {
    tokens: Vec<Tokens>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Tokens>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> f64 {
        if matches!(self.peek(0), Tokens::Fn) {
            return self.declaration();
        }
        self.statement()
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

    fn block(&mut self) -> f64 {
        let mut last_value = 0.0;
        let mut brace_count = 1;
        while brace_count > 0 && !self.is_at_end() {
            if matches!(self.peek(0), Tokens::RightBrace) {
                brace_count -= 1;
                if brace_count == 0 {
                    self.advance();
                    break;
                }
                self.advance();
                continue;
            }
            if matches!(self.peek(0), Tokens::LeftBrace) {
                brace_count += 1;
                self.advance();
                continue;
            }
            last_value = self.statement();
        }
        last_value
    }

    fn skip_block(&mut self) {
        let mut brace_count = 1;
        while brace_count > 0 && !self.is_at_end() {
            self.advance();
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

    fn consume_semicolon(&mut self, text: &str) {
        if !matches!(self.advance(), Tokens::Semicolon) {
            if !matches!(self.advance(), Tokens::Semicolon) {
                println!("Expected ';' after {}", text);
            }
        }
    }

    fn statement(&mut self) -> f64 {
        if let Tokens::Identifier(name) = self.peek(0).clone() {
            let var_name = name.clone();
            self.advance();
            if matches!(self.peek(0), Tokens::Increment) {
                self.advance();
                let mut env = GLOBAL_ENV.lock();
                if let Some(value) = env.get(&var_name) {
                    let new_value = value + 1.0;
                    env.set(var_name.to_string(), new_value);
                    drop(env);
                    self.consume_semicolon("increment operation");
                    return new_value;
                } else {
                    println!("Undefined variable: {}", var_name);
                    drop(env);
                    while !matches!(self.peek(0), Tokens::Semicolon)
                        && !matches!(self.peek(0), Tokens::EOF)
                    {
                        self.advance();
                    }
                    if matches!(self.peek(0), Tokens::Semicolon) {
                        self.advance();
                    }
                    return 0.0;
                }
            } else if matches!(self.peek(0), Tokens::Decrement) {
                self.advance();
                let mut env = GLOBAL_ENV.lock();
                if let Some(value) = env.get(&var_name) {
                    let new_value = value - 1.0;
                    env.set(var_name.to_string(), new_value);
                    drop(env);
                    self.consume_semicolon("decrement operation");
                    return new_value;
                } else {
                    println!("Undefined variable: {}", var_name);
                    drop(env);
                    while !matches!(self.peek(0), Tokens::Semicolon)
                        && !matches!(self.peek(0), Tokens::EOF)
                    {
                        self.advance();
                    }
                    if matches!(self.peek(0), Tokens::Semicolon) {
                        self.advance();
                    }
                    return 0.0;
                }
            } else {
                self.current -= 1;
            }
        }

        if matches!(self.peek(0), Tokens::Asm) {
            self.advance();
            if !matches!(self.advance(), Tokens::LeftParen) {
                println!("Expected '(' after 'asm'");
                return 0.0;
            }
            let value = match self.peek(0) {
                Tokens::String(s) => {
                    let asm_string = s.clone();
                    self.advance();
                    unsafe {
                        match asm_string.as_str() {
                            "nop" => asm!("nop"),
                            "hlt" => asm!("hlt"),
                            "cli" => asm!("cli"),
                            s if s.starts_with("mov eax, ") => {
                                if let Some(imm_str) = s.strip_prefix("mov eax, ") {
                                    if let Ok(imm) = imm_str.trim().parse::<u32>() {
                                        asm!("mov eax, {0:e}", in(reg) imm);
                                    } else {
                                        println!("Invalid immediate value for mov: {}", imm_str);
                                    }
                                }
                            }
                            s if s.starts_with("mov ebx, ") => {
                                if let Some(imm_str) = s.strip_prefix("mov ebx, ") {
                                    if let Ok(imm) = imm_str.trim().parse::<u32>() {
                                        asm!("mov ebx, {0:e}", in(reg) imm);
                                    } else {
                                        println!("Invalid immediate value for mov: {}", imm_str);
                                    }
                                }
                            }
                            s if s.starts_with("mov ecx, ") => {
                                if let Some(imm_str) = s.strip_prefix("mov ecx, ") {
                                    if let Ok(imm) = imm_str.trim().parse::<u32>() {
                                        asm!("mov ecx, {0:e}", in(reg) imm);
                                    } else {
                                        println!("Invalid immediate value for mov: {}", imm_str);
                                    }
                                }
                            }
                            s if s.starts_with("mov edx, ") => {
                                if let Some(imm_str) = s.strip_prefix("mov edx, ") {
                                    if let Ok(imm) = imm_str.trim().parse::<u32>() {
                                        asm!("mov edx, {0:e}", in(reg) imm);
                                    } else {
                                        println!("Invalid immediate value for mov: {}", imm_str);
                                    }
                                }
                            }
                            _ => println!("Unsupported asm instruction: {}", asm_string),
                        }
                    }
                    0.0
                }
                _ => 0.0,
            };
            if !matches!(self.advance(), Tokens::RightParen) {
                println!("Expected ')' after asm argument");
            }
            self.consume_semicolon("asm statement");
            return value;
        }

        if matches!(self.peek(0), Tokens::Print) {
            self.advance();
            if !matches!(self.advance(), Tokens::LeftParen) {
                println!("Expected '(' after 'print'");
                return 0.0;
            }
            let value = match self.peek(0) {
                Tokens::String(s) => {
                    let string_to_print = s.clone();
                    self.advance();
                    print!("{}", string_to_print);
                    0.0
                }
                _ => {
                    let result = self.expression();
                    print!("{}", result);
                    result
                }
            };
            if !matches!(self.advance(), Tokens::RightParen) {
                println!("Expected ')' after print argument");
            }
            self.consume_semicolon("print statement");
            return value;
        }

        if matches!(self.peek(0), Tokens::Println) {
            self.advance();
            if !matches!(self.advance(), Tokens::LeftParen) {
                println!("Expected '(' after 'println'");
                return 0.0;
            }
            let value = match self.peek(0) {
                Tokens::String(s) => {
                    let string_to_print = s.clone();
                    self.advance();
                    println!("{}", string_to_print);
                    0.0
                }
                _ => {
                    let result = self.expression();
                    println!("{}", result);
                    result
                }
            };
            if !matches!(self.advance(), Tokens::RightParen) {
                println!("Expected ')' after println argument");
            }
            self.consume_semicolon("println statement");
            return value;
        }

        if matches!(self.peek(0), Tokens::Return) {
            self.advance();
            let value = self.expression();
            self.consume_semicolon("return statement");
            return value;
        }

        if matches!(self.peek(0), Tokens::Let) {
            self.advance();
            let var_name = match self.peek(0) {
                Tokens::Identifier(name) => name.clone(),
                _ => {
                    println!("Expected identifier after 'let'");
                    return 0.0;
                }
            };
            self.advance();
            if !matches!(self.peek(0), Tokens::Assign) {
                println!("Expected '=' after variable name");
                return 0.0;
            }
            self.advance();
            let value = self.expression();
            GLOBAL_ENV.lock().set(var_name.to_string(), value);
            self.consume_semicolon("variable declaration");
            return value;
        }

        if matches!(self.peek(0), Tokens::If) {
            self.advance();
            let condition = self.expression();
            if !matches!(self.advance(), Tokens::LeftBrace) {
                println!("Expected '{{' after if condition");
                return 0.0;
            }
            let mut then_result = 0.0;
            if condition != 0.0 {
                then_result = self.block();
            } else {
                self.skip_block();
            }
            if matches!(self.peek(0), Tokens::Else) {
                self.advance();
                if !matches!(self.advance(), Tokens::LeftBrace) {
                    println!("Expected '{{' after else");
                    return 0.0;
                }
                if condition == 0.0 {
                    return self.block();
                } else {
                    self.skip_block();
                    return then_result;
                }
            }
            return then_result;
        }

        if matches!(self.peek(0), Tokens::While) {
            self.advance();
            let condition_pos = self.current;
            let mut condition = self.expression();
            if !matches!(self.advance(), Tokens::LeftBrace) {
                println!("Expected '{{' after while condition");
                return 0.0;
            }
            let body_pos = self.current;
            let mut last_value = 0.0;
            while condition != 0.0 {
                last_value = self.block();
                self.current = condition_pos;
                condition = self.expression();
                self.advance();
                self.current = body_pos;
            }
            self.skip_block();
            return last_value;
        }

        // --- FOR LOOP SUPPORT ---
        if matches!(self.peek(0), Tokens::For) {
            self.advance();
            if !matches!(self.advance(), Tokens::LeftParen) {
                println!("Expected '(' after 'for'");
                return 0.0;
            }
            self.statement(); // Initialization
            if !matches!(self.peek(0), Tokens::Semicolon) {
                println!("Expected ';' after for-init");
                return 0.0;
            }
            self.advance();
            let cond_pos = self.current;
            let mut condition = self.expression(); // Condition
            if !matches!(self.peek(0), Tokens::Semicolon) {
                println!("Expected ';' after for-condition");
                return 0.0;
            }
            self.advance();
            let incr_pos = self.current;
            self.statement(); // Increment
            if !matches!(self.peek(0), Tokens::RightParen) {
                println!("Expected ')' after for-increment");
                return 0.0;
            }
            self.advance();
            if !matches!(self.advance(), Tokens::LeftBrace) {
                println!("Expected '{{' after for loop header");
                return 0.0;
            }
            let body_pos = self.current;
            let mut last_value = 0.0;
            while condition != 0.0 {
                last_value = self.block();
                self.current = incr_pos;
                self.statement(); // Increment
                self.current = cond_pos;
                condition = self.expression();
                self.current = body_pos;
            }
            self.skip_block();
            return last_value;
        }

        if let Tokens::Identifier(name) = self.peek(0).clone() {
            if let Tokens::Assign = self.peek(1) {
                if GLOBAL_ENV.lock().get(&name).is_none() {
                    println!(
                        "Error: Variable '{}' must be declared with 'let' before assignment",
                        name
                    );
                    self.advance();
                    self.advance();
                    let _ = self.expression();
                    self.consume_semicolon("assignment statement");
                    return 0.0;
                }
            }
        }

        let result = self.expression();
        self.consume_semicolon("expression");
        result
    }

    fn declaration(&mut self) -> f64 {
        if matches!(self.peek(0), Tokens::Fn) {
            self.advance();
            let fn_name = match self.peek(0) {
                Tokens::Identifier(name) => name.clone(),
                _ => {
                    println!("Expected function name after 'fn'");
                    return 0.0;
                }
            };
            self.advance();
            if !matches!(self.advance(), Tokens::LeftParen) {
                println!("Expected '(' after function name");
                return 0.0;
            }
            let mut params = Vec::new();
            if !matches!(self.peek(0), Tokens::RightParen) {
                loop {
                    if let Tokens::Identifier(param_name) = self.peek(0).clone() {
                        params.push(param_name);
                        self.advance();
                        if matches!(self.peek(0), Tokens::Comma) {
                            self.advance();
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
            }
            self.advance();
            if !matches!(self.advance(), Tokens::LeftBrace) {
                println!("Expected '{{' after function parameters");
                return 0.0;
            }
            let body_start = self.current;
            let mut brace_count = 1;
            let mut body_tokens = Vec::new();
            while brace_count > 0 && !self.is_at_end() {
                let token = self.peek(0).clone();
                body_tokens.push(token.clone());
                self.advance();
                if matches!(token, Tokens::LeftBrace) {
                    brace_count += 1;
                } else if matches!(token, Tokens::RightBrace) {
                    brace_count -= 1;
                    if brace_count == 0 {
                        body_tokens.pop();
                        break;
                    }
                }
            }
            let function = Function {
                name: fn_name.clone(),
                parameters: params,
                body_tokens,
                body_start,
                body_end: self.current - 1,
            };
            FUNCTION_REGISTRY.lock().insert(fn_name, function);
            return 0.0;
        }
        self.statement()
    }

    fn expression(&mut self) -> f64 {
        if let Tokens::Identifier(name) = self.peek(0).clone() {
            if let Tokens::Assign = self.peek(1) {
                if GLOBAL_ENV.lock().get(&name).is_none() {
                    self.advance();
                    self.advance();
                    let value = self.expression();
                    return value;
                } else {
                    let var_name = name;
                    self.advance();
                    self.advance();
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
            self.advance();
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
                if matches!(self.peek(0), Tokens::LeftParen) {
                    self.advance();
                    let mut args = Vec::new();
                    if !matches!(self.peek(0), Tokens::RightParen) {
                        loop {
                            args.push(self.expression());
                            if matches!(self.peek(0), Tokens::Comma) {
                                self.advance();
                            } else if matches!(self.peek(0), Tokens::RightParen) {
                                break;
                            } else {
                                println!("Expected ',' or ')' after argument");
                                return 0.0;
                            }
                        }
                    }
                    self.advance();
                    return self.call_function(&name, args);
                }
                match GLOBAL_ENV.lock().get(&name) {
                    Some(value) => value,
                    None => {
                        println!("Undefined variable: {}", name);
                        0.0
                    }
                }
            }
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

    fn call_function(&mut self, name: &str, args: Vec<f64>) -> f64 {
        let function_opt = FUNCTION_REGISTRY.lock().get(name).cloned();
        if let Some(function) = function_opt {
            let global_env_backup = GLOBAL_ENV.lock().clone();
            let mut function_env = Environment::new();
            for (key, value) in &global_env_backup.variables {
                function_env.set(key.clone(), *value);
            }
            for (i, param_name) in function.parameters.iter().enumerate() {
                if i < args.len() {
                    function_env.set(param_name.clone(), args[i]);
                } else {
                    println!("Warning: Missing argument for parameter '{}'", param_name);
                    function_env.set(param_name.clone(), 0.0);
                }
            }
            *GLOBAL_ENV.lock() = function_env;
            let current_pos = self.current;
            let mut function_parser = Parser::new(function.body_tokens.clone());
            let result = function_parser.parse();
            *GLOBAL_ENV.lock() = global_env_backup;
            self.current = current_pos;
            return result;
        } else {
            println!("Undefined function: {}", name);
            0.0
        }
    }
}