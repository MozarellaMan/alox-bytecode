use crate::{
    chunk::Chunk,
    scanner::Scanner,
    token::{Token, TokenKind},
};

pub type CompilationResult = Result<(), CompilationError>;

pub struct Parser<'source> {
    scanner: Scanner<'source>,
    current: Option<Token<'source>>,
    previous: Option<Token<'source>>,
    hadError: bool,
    panicMode: bool,
}

impl<'a> Parser<'a> {
    fn new(scanner: Scanner<'a>) -> Self {
        Self {
            scanner,
            current: None,
            previous: None,
            hadError: false,
            panicMode: false,
        }
    }

    pub fn compile(source: &'a str, chunk: &mut Chunk) -> CompilationResult {
        let scanner = Scanner::new(source);
        let mut parser = Parser::new(scanner);
        parser.advance();
        parser.expression();
        parser.consume(TokenKind::Eof, "Expected end of expression.");
        if parser.hadError {
            Err(CompilationError::Error)
        } else {
            Ok(())
        }
    }

    fn advance(&mut self) {
        self.previous = self.current.take();
        loop {
            self.current = Some(self.scanner.scan_token());
            if self.current.as_ref().unwrap().kind != TokenKind::Error {
                break;
            }
            self.error_at_current("")
        }
    }

    fn expression(&mut self) {}

    fn consume(&mut self, token_kind: TokenKind, error_msg: &str) {
        if let Some(token) = self.current.as_ref() {
            if token.kind == token_kind {
                self.advance();
                return;
            }
        }
        self.error_at_current("");
    }

    

    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.current.clone(), message);
    }

    fn error_at(&mut self, token: Option<Token>, message: &str) {
        if self.panicMode {
            return;
        }
        if let Some(token) = token {
            eprint!("[line {}] Error", token.line);
            match token.kind {
                TokenKind::Eof => eprint!(" at end"),
                TokenKind::Error => {}
                _ => eprint!(" at {}", token.lexeme),
            }
            eprint!(": {}", message);
        } else {
            eprintln!("Parser error.");
        }
    }
}

#[derive(Debug)]
pub enum CompilationError {
    Error,
}
