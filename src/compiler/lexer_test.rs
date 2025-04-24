#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex() {
        let input = r#"
            let x = 42;
            fn add(a, b) {
                return a + b;
            }
        "#;

        let tokens = lex(input);
        assert_eq!(
            tokens,
            vec![
                Token::Let,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Number(42.0),
                Token::Semicolon,
                Token::Fn,
                Token::Identifier("add".to_string()),
                Token::OpenParen,
                Token::Identifier("a".to_string()),
                Token::Comma,
                Token::Identifier("b".to_string()),
                Token::CloseParen,
                Token::OpenBrace,
                Token::Identifier("return".to_string()),
                Token::Identifier("a".to_string()),
                Token::Plus,
                Token::Identifier("b".to_string()),
                Token::Semicolon,
                Token::CloseBrace,
                Token::EOF,
            ]
        );
    }
}