use crate::token::{Token, TokenKind};

pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: usize,
}

impl<'source> Scanner<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan(&mut self) {
        loop {
            let line = 0;
            let token = self.scan_token();
            if token.line != line || token.line == 0 {
                print!("{}", token.line);
                // line = token.line;
            } else {
                print!("   | ");
            }
            println!(" {:?} '{}'", token.kind, token.lexeme);
            if let TokenKind::Eof = token.kind {
                break;
            }
        }
    }

    pub fn scan_token(&mut self) -> Token<'source> {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenKind::Eof);
        }

        let char = self.advance();

        if char.is_ascii_digit() {
            return self.number();
        }

        if is_alpha(char) {
            return self.identifier();
        }

        match char {
            b'(' => self.make_token(TokenKind::LeftParen),
            b')' => self.make_token(TokenKind::RightParen),
            b'{' => self.make_token(TokenKind::LeftBrace),
            b'}' => self.make_token(TokenKind::RightBrace),
            b';' => self.make_token(TokenKind::Semicolon),
            b',' => self.make_token(TokenKind::Comma),
            b'.' => self.make_token(TokenKind::Dot),
            b'-' => self.make_token(TokenKind::Minus),
            b'+' => self.make_token(TokenKind::Plus),
            b'/' => self.make_token(TokenKind::Slash),
            b'*' => self.make_token(TokenKind::Star),
            b'!' => self.match_next_token(b'=', TokenKind::BangEqual, TokenKind::Bang),
            b'=' => self.match_next_token(b'=', TokenKind::EqualEqual, TokenKind::Equal),
            b'<' => self.match_next_token(b'=', TokenKind::LessEqual, TokenKind::Less),
            b'>' => self.match_next_token(b'=', TokenKind::GreaterEqual, TokenKind::Greater),
            b'"' => self.string(),
            _ => Token::error("Unexpected character.", self.line),
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            let char = self.peek();
            if char == b'\n' {
                self.line += 1;
                self.advance();
            } else if char == b'/' {
                if self.peek_next() == b'/' {
                    while self.peek() != b'\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    break;
                }
            } else if char.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() {
            b'\0'
        } else {
            self.source.as_bytes()[self.current]
        }
    }

    fn peek_next(&self) -> u8 {
        if self.is_at_end() || self.current + 1 >= self.source.len() {
            return b'\0';
        }
        self.source.as_bytes()[self.current + 1]
    }

    fn advance(&mut self) -> u8 {
        self.current += 1;
        self.source.as_bytes()[self.current - 1]
    }

    fn match_next(&mut self, expected: u8) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.as_bytes()[self.current] != expected {
            return false;
        }
        self.current += 1;
        true
    }

    /// Checks the next token, if matches makes the token kind specified by opt1, op2 otherwise
    fn match_next_token(
        &mut self,
        expected: u8,
        opt1: TokenKind,
        opt2: TokenKind,
    ) -> Token<'source> {
        if self.match_next(expected) {
            self.make_token(opt1)
        } else {
            self.make_token(opt2)
        }
    }

    fn make_token(&self, kind: TokenKind) -> Token<'source> {
        Token::make(
            self.source,
            kind,
            self.start,
            self.current_token_length(),
            self.line,
        )
    }

    fn identifier(&mut self) -> Token<'source> {
        while is_alpha(self.peek()) || self.peek().is_ascii_digit() {
            self.advance();
        }
        self.make_token(self.identifier_kind())
    }

    fn identifier_kind(&self) -> TokenKind {
        match self.source.as_bytes()[self.start] {
            b'a' => self.check_keyword(1, 2, "nd", TokenKind::And),
            b'c' => self.check_keyword(1, 4, "lass", TokenKind::Class),
            b'e' => self.check_keyword(1, 3, "lse", TokenKind::Else),
            b'i' => self.check_keyword(1, 1, "f", TokenKind::If),
            b'n' => self.check_keyword(1, 2, "il", TokenKind::Nil),
            b'o' => self.check_keyword(1, 1, "r", TokenKind::Or),
            b'p' => self.check_keyword(1, 4, "rint", TokenKind::Print),
            b'r' => self.check_keyword(1, 5, "eturn", TokenKind::Return),
            b's' => self.check_keyword(1, 4, "uper", TokenKind::Super),
            b'v' => self.check_keyword(1, 2, "ar", TokenKind::Var),
            b'w' => self.check_keyword(1, 4, "hile", TokenKind::While),
            b'f' => {
                if self.current_token_length() > 1 {
                    match self.source.as_bytes()[self.start + 1] {
                        b'a' => return self.check_keyword(2, 3, "lse", TokenKind::False),
                        b'o' => return self.check_keyword(2, 1, "r", TokenKind::For),
                        b'u' => return self.check_keyword(2, 1, "n", TokenKind::Fun),
                        _ => {}
                    }
                }
                TokenKind::Identifier
            }
            b't' => {
                if self.current_token_length() > 1 {
                    match self.source.as_bytes()[self.start + 1] {
                        b'h' => return self.check_keyword(2, 2, "is", TokenKind::This),
                        b'r' => return self.check_keyword(2, 2, "ue", TokenKind::True),
                        _ => {}
                    }
                }
                TokenKind::Identifier
            }
            _ => TokenKind::Identifier,
        }
    }

    fn check_keyword(&self, start: usize, length: usize, rest: &str, kind: TokenKind) -> TokenKind {
        let end = self.start + start + length;
        if self.current_token_length() == start + length
            && self.source[self.start + start..end] == *rest
        {
            kind
        } else {
            TokenKind::Identifier
        }
    }

    fn number(&mut self) -> Token<'source> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == b'.' && self.peek_next().is_ascii_digit() {
            // consume the "."
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.make_token(TokenKind::Number)
    }

    fn string(&mut self) -> Token<'source> {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Token::error("Unterminated string.", self.line);
        }

        // closing quote
        self.advance();

        self.make_token(TokenKind::String)
    }

    fn current_token_length(&self) -> usize {
        self.current - self.start
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

fn is_alpha(char: u8) -> bool {
    char.is_ascii_alphabetic() || char == b'_'
}
