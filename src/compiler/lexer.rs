#[derive(Debug, PartialEq)]
pub enum Token {
    Let,                // `let` keyword
    Fn,                 // `fn` keyword
    Identifier(String), // Variable or function names
    Number(f64),        // Numeric literals
    String(String),     // String literals
    Plus,               // `+` operator
    Minus,              // `-` operator
    Multiply,           // `*` operator
    Divide,             // `/` operator
    Equals,             // `=` operator
    OpenParen,          // `(` symbol
    CloseParen,         // `)` symbol
    OpenBrace,          // `{` symbol
    CloseBrace,         // `}` symbol
    Semicolon,          // `;` symbol
    Colon,              // `:` symbol
    Comma,              // `,` symbol
    EOF,                // End of file
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' => {
                chars.next(); // Skip whitespace
            }
            'a'..='z' | 'A'..='Z' => {
                let mut identifier = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        identifier.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(match identifier.as_str() {
                    "let" => Token::Let,
                    "fn" => Token::Fn,
                    _ => Token::Identifier(identifier),
                });
            }
            '0'..='9' => {
                let mut number = String::new();
                while let Some(&digit) = chars.peek() {
                    if digit.is_numeric() || digit == '.' {
                        number.push(digit);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(number.parse().unwrap()));
            }
            '"' => {
                chars.next(); // Skip opening quote
                let mut string = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        chars.next(); // Skip closing quote
                        break;
                    } else {
                        string.push(c);
                        chars.next();
                    }
                }
                tokens.push(Token::String(string));
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Multiply);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Divide);
                chars.next();
            }
            '=' => {
                tokens.push(Token::Equals);
                chars.next();
            }
            '(' => {
                tokens.push(Token::OpenParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::CloseParen);
                chars.next();
            }
            '{' => {
                tokens.push(Token::OpenBrace);
                chars.next();
            }
            '}' => {
                tokens.push(Token::CloseBrace);
                chars.next();
            }
            ';' => {
                tokens.push(Token::Semicolon);
                chars.next();
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            _ => panic!("Unexpected character: {}", ch),
        }
    }

    tokens.push(Token::EOF);
    tokens
}