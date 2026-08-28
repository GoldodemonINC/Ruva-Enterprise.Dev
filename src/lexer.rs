use crate::ast::{Span, Token};
use anyhow::{bail, Result};

pub struct Lexer<'a> {
    _source: &'a str,
    bytes: Vec<u8>,
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            _source: source,
            bytes: source.bytes().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn peek_ahead(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let b = self.bytes.get(self.pos).copied()?;
        self.pos += 1;
        if b == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(b)
    }

    fn skip_whitespace(&mut self) {
        while let Some(b) = self.peek() {
            if b.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(b) = self.peek() {
            if b == b'\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> Result<()> {
        let mut depth = 1u32;
        while depth > 0 {
            match (self.peek(), self.peek_ahead(1)) {
                (Some(b'/'), Some(b'*')) => {
                    self.advance();
                    self.advance();
                    depth += 1;
                }
                (Some(b'*'), Some(b'/')) => {
                    self.advance();
                    self.advance();
                    depth -= 1;
                }
                (Some(_), _) => {
                    self.advance();
                }
                (None, _) => {
                    bail!("Unterminated block comment at line {}", self.line);
                }
            }
        }
        Ok(())
    }

    fn read_string(&mut self, quote: u8) -> Result<String> {
        let mut s = String::new();
        loop {
            match self.advance() {
                Some(b'\\') => {
                    match self.advance() {
                        Some(b'n') => s.push('\n'),
                        Some(b't') => s.push('\t'),
                        Some(b'r') => s.push('\r'),
                        Some(b'\\') => s.push('\\'),
                        Some(b'0') => s.push('\0'),
                        Some(c) if c == quote => s.push(c as char),
                        Some(c) => {
                            s.push('\\');
                            s.push(c as char);
                        }
                        None => bail!("Unterminated string escape at line {}", self.line),
                    }
                }
                Some(c) if c == quote => break,
                Some(c) => s.push(c as char),
                None => bail!("Unterminated string at line {}", self.line),
            }
        }
        Ok(s)
    }

    fn read_number(&mut self, first: u8) -> Token {
        let mut num_str = String::new();
        num_str.push(first as char);

        let mut is_float = false;

        // hex
        if first == b'0' && self.peek() == Some(b'x') {
            num_str.push(self.advance().unwrap() as char);
            while let Some(b) = self.peek() {
                if b.is_ascii_hexdigit() || b == b'_' {
                    num_str.push(self.advance().unwrap() as char);
                } else {
                    break;
                }
            }
            let cleaned: String = num_str.chars().filter(|c| *c != '_').collect();
            let val = u64::from_str_radix(&cleaned[2..], 16).unwrap_or(0) as i64;
            return Token::Int(val);
        }

        // binary
        if first == b'0' && self.peek() == Some(b'b') {
            num_str.push(self.advance().unwrap() as char);
            while let Some(b) = self.peek() {
                if b == b'0' || b == b'1' || b == b'_' {
                    num_str.push(self.advance().unwrap() as char);
                } else {
                    break;
                }
            }
            let cleaned: String = num_str.chars().filter(|c| *c != '_').collect();
            let val = u64::from_str_radix(&cleaned[2..], 2).unwrap_or(0) as i64;
            return Token::Int(val);
        }

        while let Some(b) = self.peek() {
            if b.is_ascii_digit() || b == b'_' {
                num_str.push(self.advance().unwrap() as char);
            } else if b == b'.' && !is_float && self.peek_ahead(1) != Some(b'.') {
                is_float = true;
                num_str.push(self.advance().unwrap() as char);
            } else if b == b'f' || b == b'u' || b == b'i' {
                // type suffix — skip it
                self.advance();
                break;
            } else {
                break;
            }
        }

        let cleaned: String = num_str.chars().filter(|c| *c != '_').collect();

        if is_float {
            Token::Float(cleaned.parse::<f64>().unwrap_or(0.0))
        } else {
            Token::Int(cleaned.parse::<i64>().unwrap_or(0))
        }
    }

    fn read_identifier(&mut self, first: u8) -> Token {
        let mut ident = String::new();
        ident.push(first as char);

        while let Some(b) = self.peek() {
            if b.is_ascii_alphanumeric() || b == b'_' {
                ident.push(self.advance().unwrap() as char);
            } else {
                break;
            }
        }

        match ident.as_str() {
            "fn" => Token::Fn,
            "let" => Token::Let,
            "mut" => Token::Mut,
            "pub" => Token::Pub,
            "struct" => Token::Struct,
            "class" => Token::Class,
            "impl" => Token::Impl,
            "trait" => Token::Trait,
            "enum" => Token::Enum,
            "type" => Token::Type,
            "if" => Token::If,
            "else" => Token::Else,
            "for" => Token::For,
            "while" => Token::While,
            "loop" => Token::Loop,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "return" => Token::Return,
            "match" => Token::Match,
            "self" => Token::Self_,
            "Self" => Token::SelfType,
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "null" => Token::Null,
            "import" => Token::Import,
            "use" => Token::Use,
            "as" => Token::As,
            "move" => Token::Move,
            "unsafe" => Token::Unsafe,
            "extern" => Token::Extern,
            "static" => Token::Static,
            "const" => Token::Const,
            "where" => Token::Where,
            "test" => Token::Test,
            "catch" => Token::Catch,
            "in" => Token::In,
            "mod" => Token::Mod,
            "assert" => Token::Ident("assert".into()),
            "expect" => Token::Ident("expect".into()),
            "print" => Token::Ident("print".into()),
            "interface" => Token::Interface,
            "abstract" => Token::Abstract,
            "synchronized" => Token::Synchronized,
            "package" => Token::Package,
            "try" => Token::Try,
            "finally" => Token::Finally,
            "throw" => Token::Throw,
            "comptime" => Token::Comptime,
            _ => Token::Ident(ident),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<(Token, Span)>> {
        // Pre-allocate: roughly 1 token per 8 bytes of source
        let estimated = (self.bytes.len() / 8).max(32);
        let mut tokens = Vec::with_capacity(estimated);

        loop {
            self.skip_whitespace();

            let span = Span {
                line: self.line,
                col: self.col,
            };

            let Some(ch) = self.peek() else {
                tokens.push((Token::Eof, span));
                break;
            };

            // Comments
            if ch == b'/' && self.peek_ahead(1) == Some(b'/') {
                self.advance();
                self.advance();
                self.skip_line_comment();
                continue;
            }
            if ch == b'/' && self.peek_ahead(1) == Some(b'*') {
                self.advance();
                self.advance();
                self.skip_block_comment()?;
                continue;
            }

            // FString: f"Hello {name}"
            if ch == b'f' && self.peek_ahead(1) == Some(b'"') {
                self.advance(); // skip 'f'
                self.advance(); // skip opening quote
                let mut parts = Vec::new();
                let mut current = String::new();
                loop {
                    match self.advance() {
                        Some(b'"') => {
                            if !current.is_empty() {
                                parts.push((current.clone(), false));
                                current.clear();
                            }
                            break;
                        }
                        Some(b'{') => {
                            if !current.is_empty() {
                                parts.push((current.clone(), false));
                                current.clear();
                            }
                            // Read until matching }
                            let mut depth = 1u32;
                            let mut expr = String::new();
                            while depth > 0 {
                                match self.advance() {
                                    Some(b'{') => { depth += 1; expr.push('{'); }
                                    Some(b'}') => { depth -= 1; if depth > 0 { expr.push('}'); } }
                                    Some(c) => expr.push(c as char),
                                    None => bail!("Unterminated f-string expression at {}:{}", self.line, self.col),
                                }
                            }
                            parts.push((expr, true));
                        }
                        Some(b'\\') => {
                            match self.advance() {
                                Some(b'n') => current.push('\n'),
                                Some(b't') => current.push('\t'),
                                Some(b'r') => current.push('\r'),
                                Some(b'\\') => current.push('\\'),
                                Some(b'{') => current.push('{'),
                                Some(b'}') => current.push('}'),
                                Some(c) => { current.push('\\'); current.push(c as char); }
                                None => bail!("Unterminated f-string at {}:{}", self.line, self.col),
                            }
                        }
                        Some(c) => current.push(c as char),
                        None => bail!("Unterminated f-string at {}:{}", self.line, self.col),
                    }
                }
                // Convert parts to tokens: FStringStart, text parts, expr parts, FStringEnd
                // For simplicity, we'll generate a format! macro call
                tokens.push((Token::FStringStart, span));
                for (text, is_expr) in parts {
                    if is_expr {
                        tokens.push((Token::FStringExpr, span));
                        // Tokenize the expression inside the braces
                        let inner_tokens = Lexer::new(&text).tokenize()?;
                        for (t, s) in inner_tokens {
                            if t != Token::Eof {
                                tokens.push((t, s));
                            }
                        }
                        tokens.push((Token::RBrace, span));
                    } else {
                        tokens.push((Token::FStringPart(text), span));
                    }
                }
                tokens.push((Token::FStringEnd, span));
                continue;
            }

            // String literal
            if ch == b'"' || ch == b'\'' {
                let quote = ch;
                self.advance();

                // Char literal
                if ch == b'\'' {
                    let c = match self.advance() {
                        Some(b'\\') => match self.advance() {
                            Some(b'n') => '\n',
                            Some(b't') => '\t',
                            Some(b'r') => '\r',
                            Some(b'\\') => '\\',
                            Some(b'0') => '\0',
                            Some(c) => c as char,
                            None => bail!("Unterminated char escape"),
                        },
                        Some(c) => c as char,
                        None => bail!("Unterminated char literal"),
                    };
                    if self.peek() != Some(b'\'') {
                        bail!("Expected closing ' for char literal");
                    }
                    self.advance();
                    tokens.push((Token::Char(c), span));
                    continue;
                }

                // String literal
                let s = self.read_string(quote)?;
                tokens.push((Token::Str(s), span));
                continue;
            }

            // Number literal
            if ch.is_ascii_digit() {
                self.advance(); // consume the first digit
                let token = self.read_number(ch);
                tokens.push((token, span));
                continue;
            }

            // Identifier or keyword
            if ch.is_ascii_alphabetic() || ch == b'_' {
                self.advance(); // consume the first char
                let token = self.read_identifier(ch);
                tokens.push((token, span));
                continue;
            }

            // Operators and delimiters
            self.advance();
            let token = match ch {
                b'+' => match self.peek() {
                    Some(b'=') => { self.advance(); Token::PlusEq }
                    _ => Token::Plus,
                },
                b'-' => match self.peek() {
                    Some(b'=') => { self.advance(); Token::MinusEq }
                    Some(b'>') => { self.advance(); Token::Arrow }
                    _ => Token::Minus,
                },
                b'*' => match self.peek() {
                    Some(b'=') => { self.advance(); Token::StarEq }
                    _ => Token::Star,
                },
                b'/' => match self.peek() {
                    Some(b'=') => { self.advance(); Token::SlashEq }
                    _ => Token::Slash,
                },
                b'%' => Token::Percent,
                b'=' => match self.peek() {
                    Some(b'=') => { self.advance(); Token::EqEq }
                    Some(b'>') => { self.advance(); Token::FatArrow }
                    _ => Token::Eq,
                },
                b'!' => match self.peek() {
                    Some(b'=') => { self.advance(); Token::Ne }
                    _ => Token::Not,
                },
                b'<' => match self.peek() {
                    Some(b'=') => { self.advance(); Token::Le }
                    Some(b'<') => { self.advance(); Token::Shl }
                    _ => Token::Lt,
                },
                b'>' => match self.peek() {
                    Some(b'=') => { self.advance(); Token::Ge }
                    Some(b'>') => { self.advance(); Token::Shr }
                    _ => Token::Gt,
                },
                b'&' => match self.peek() {
                    Some(b'&') => { self.advance(); Token::And }
                    Some(b'=') => { self.advance(); Token::AmpEq }
                    _ => Token::Amp,
                },
                b'|' => match self.peek() {
                    Some(b'|') => { self.advance(); Token::Or }
                    Some(b'=') => { self.advance(); Token::PipeEq }
                    _ => Token::Pipe,
                },
                b'^' => match self.peek() {
                    Some(b'=') => { self.advance(); Token::CaretEq }
                    _ => Token::Caret,
                },
                b'~' => Token::Tilde,
                b'.' => match self.peek() {
                    Some(b'.') => {
                        self.advance();
                        if self.peek() == Some(b'=') {
                            self.advance();
                            Token::DotDotEq
                        } else {
                            Token::DotDot
                        }
                    }
                    _ => Token::Dot,
                },
                b':' => match self.peek() {
                    Some(b':') => { self.advance(); Token::DoubleColon }
                    _ => Token::Colon,
                },
                b'(' => Token::LParen,
                b')' => Token::RParen,
                b'{' => Token::LBrace,
                b'}' => Token::RBrace,
                b'[' => Token::LBracket,
                b']' => Token::RBracket,
                b';' => Token::Semicolon,
                b',' => Token::Comma,
                b'#' => Token::Hash,
                b'@' => Token::At,
                b'?' => match self.peek() {
                    Some(b'.') => { self.advance(); Token::QuestionDot }
                    Some(b'?') => { self.advance(); Token::NullCoalesce }
                    _ => Token::Question,
                }
                b'_' => Token::Underscore,
                _ => bail!("Unexpected character '{}' at {}:{}",
                    ch as char, span.line, span.col),
            };

            tokens.push((token, span));
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("let x = 42;");
        let tokens: Vec<Token> = lexer
            .tokenize()
            .unwrap()
            .into_iter()
            .map(|(t, _)| t)
            .collect();

        assert_eq!(
            tokens,
            vec![
                Token::Let,
                Token::Ident("x".into()),
                Token::Eq,
                Token::Int(42),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new(r#""hello world""#);
        let tokens: Vec<Token> = lexer
            .tokenize()
            .unwrap()
            .into_iter()
            .map(|(t, _)| t)
            .collect();

        assert_eq!(
            tokens,
            vec![Token::Str("hello world".into()), Token::Eof,]
        );
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("a == b && c != d");
        let tokens: Vec<Token> = lexer
            .tokenize()
            .unwrap()
            .into_iter()
            .map(|(t, _)| t)
            .collect();

        assert_eq!(
            tokens,
            vec![
                Token::Ident("a".into()),
                Token::EqEq,
                Token::Ident("b".into()),
                Token::And,
                Token::Ident("c".into()),
                Token::Ne,
                Token::Ident("d".into()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("let x = 1; // comment\nlet y = 2;");
        let tokens: Vec<Token> = lexer
            .tokenize()
            .unwrap()
            .into_iter()
            .map(|(t, _)| t)
            .collect();

        assert_eq!(
            tokens,
            vec![
                Token::Let,
                Token::Ident("x".into()),
                Token::Eq,
                Token::Int(1),
                Token::Semicolon,
                Token::Let,
                Token::Ident("y".into()),
                Token::Eq,
                Token::Int(2),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }
}
