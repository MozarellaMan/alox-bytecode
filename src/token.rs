#[derive(Debug, Clone)]
pub struct Token<'source> {
    pub kind: TokenKind,
    pub line: usize,
    pub lexeme: &'source str,
}

impl<'source> Token<'source> {
    pub fn make(
        source: &'source str,
        kind: TokenKind,
        start: usize,
        length: usize,
        line: usize,
    ) -> Self {
        let end = start + length;
        Self {
            kind,
            line,
            lexeme: &source[start..end],
        }
    }

    pub fn error(msg: &'source str, line: usize) -> Self {
        Self {
            kind: TokenKind::Error,
            line,
            lexeme: msg,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // single char tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // one or two char tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Print,

    Eof,
    Error,
}
