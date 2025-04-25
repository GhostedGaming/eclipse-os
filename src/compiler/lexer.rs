extern crate alloc;
use alloc::vec::Vec;

#[derive(Debug, PartialEq)]
pub enum Tokens {
    // Components
    Let,
    Fn,
    Identifier(String),
    Number(f64),
    String(String),
    Plus,
    Minus,
    Multiply,
    Divide,
    Equals,
    OpenParan,
    CloseParan,
    OpenBrace,
    CloseBrace,
    Semicolon,
    Colon,
    Comma,
    EOF,

    // Statements
    If,
    Else,
    While,
    For,
    Break,
    Continue,
    Struct,
    Enum,
    Mut,

    // Operators
    EqualsEquals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterEquals,
    LessEquals,
    And,
    Or,
    Not,
    PlusEquals,
    MinusEquals,
    MultiplyEquals,
    DivideEquals,

    // Misc
    OpenBracket,
    CloseBracket,

    Dot,
}

const example1: &str = "
    fn main() {
        let var1 = 1;
        let var2 = 2;

        print(\"{}\" var1+var2)
    }
";

fn lexer(src: &str) -> Result<(Vec<Tokens>, &str), (Vec<Tokens>, &str)> {
    let mut src_bytes = src.as_bytes();
    let mut tokens = Vec::new();

    'tokenizing:loop {
        src_bytes = match src_bytes {
            [b'f',b'n', b' ', rest@..] => {
                tokens.push(Tokens::Fn);
                rest
            }
            [b'/',b'/', rest@..] => {
                let mut rest = rest;
                'ignoring_comment:loop {
                    let [first, rest1@..] = rest else {
                        break 'tokenizing
                    };

                    rest = rest1;

                    if first == b'\n' {
                        continue 'tokenizing
                    } else {
                        continue 'ignoring_comment
                    }
                }
            }
        }
    }
}