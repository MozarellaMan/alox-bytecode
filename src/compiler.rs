use std::{convert::TryInto, u8};

use crate::{
    chunk::{Chunk, Value},
    opcodes::Op,
    scanner::Scanner,
    token::{Token, TokenKind},
};

pub type CompilationResult = Result<(), CompilationError>;

pub struct Parser<'source, 'chunk> {
    scanner: Scanner<'source>,
    current: Option<Token<'source>>,
    previous: Option<Token<'source>>,
    current_chunk: &'chunk mut Chunk,
    had_error: bool,
    panic_mode: bool,
}

impl<'source, 'chunk> Parser<'source, 'chunk> {
    fn new(scanner: Scanner<'source>, chunk: &'chunk mut Chunk) -> Self {
        Self {
            scanner,
            current: None,
            previous: None,
            had_error: false,
            panic_mode: false,
            current_chunk: chunk,
        }
    }

    pub fn compile(source: &'source str, chunk: &mut Chunk) -> CompilationResult {
        let scanner = Scanner::new(source);
        let mut parser = Parser::new(scanner, chunk);
        parser.advance();
        parser.expression();
        parser.consume(TokenKind::Eof, "Expected end of expression.");
        if parser.had_error {
            Err(CompilationError::Error)
        } else {
            parser.end_compiler();
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

    fn previous_token(&self) -> &Token {
        if let Some(previous) = &self.previous {
            previous
        } else {
            panic!("No previous tokens!")
        }
    }

    fn current_token(&self) -> &Token {
        if let Some(current) = &self.current {
            current
        } else {
            panic!("No previous tokens!")
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        let value = self.previous_token().lexeme.parse::<f64>().unwrap();
        self.emit_constant(value);
    }

    fn unary(&mut self) {
        let op_kind = self.previous_token().kind;

        // compile operand
        self.parse_precedence(Precedence::Unary);

        // emit op instruction
        match op_kind {
            TokenKind::Minus => self.emit_byte(Op::Negate.u8()),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self) {
        let op_kind = self.previous_token().kind;
        let rule = self.get_rule(op_kind);
        self.parse_precedence((rule.precedence as u8 + 1).into());

        match op_kind {
            TokenKind::Plus => self.emit_byte(Op::Add.u8()),
            TokenKind::Minus => self.emit_byte(Op::Subract.u8()),
            TokenKind::Star => self.emit_byte(Op::Multiply.u8()),
            TokenKind::Slash => self.emit_byte(Op::Divide.u8()),
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self.get_rule(self.previous_token().kind).prefix;

        if let Some(rule) = prefix_rule {
            rule(self);
        } else {
            self.error("Expected expression.");
            return;
        }

        while precedence as u8 <= self.get_rule(self.current_token().kind).precedence as u8 {
            self.advance();
            let infix_rule = self.get_rule(self.previous_token().kind).infix;
            if let Some(infix) = infix_rule {
                infix(self)
            }
        }
    }

    fn get_rule(&mut self, op_kind: TokenKind) -> ParseRule {
        match op_kind {
            TokenKind::LeftParen => {
                ParseRule::new(Some(|this| this.grouping()), None, Precedence::None)
            }
            TokenKind::RightParen => ParseRule::none(),
            TokenKind::LeftBrace => ParseRule::none(),
            TokenKind::RightBrace => ParseRule::none(),
            TokenKind::Comma => ParseRule::none(),
            TokenKind::Dot => ParseRule::none(),
            TokenKind::Minus => ParseRule::new(
                Some(|this| this.unary()),
                Some(|this| this.binary()),
                Precedence::Term,
            ),
            TokenKind::Plus => ParseRule::new(None, Some(|this| this.binary()), Precedence::Term),
            TokenKind::Semicolon => ParseRule::none(),
            TokenKind::Slash => {
                ParseRule::new(None, Some(|this| this.binary()), Precedence::Factor)
            }
            TokenKind::Star => ParseRule::new(None, Some(|this| this.binary()), Precedence::Factor),
            TokenKind::Bang => ParseRule::none(),
            TokenKind::BangEqual => ParseRule::none(),
            TokenKind::Equal => ParseRule::none(),
            TokenKind::EqualEqual => ParseRule::none(),
            TokenKind::Greater => ParseRule::none(),
            TokenKind::GreaterEqual => ParseRule::none(),
            TokenKind::Less => ParseRule::none(),
            TokenKind::LessEqual => ParseRule::none(),
            TokenKind::Identifier => ParseRule::none(),
            TokenKind::String => ParseRule::none(),
            TokenKind::Number => ParseRule::new(Some(|this| this.number()), None, Precedence::None),
            TokenKind::And => ParseRule::none(),
            TokenKind::Class => ParseRule::none(),
            TokenKind::Else => ParseRule::none(),
            TokenKind::False => ParseRule::none(),
            TokenKind::Fun => ParseRule::none(),
            TokenKind::For => ParseRule::none(),
            TokenKind::If => ParseRule::none(),
            TokenKind::Nil => ParseRule::none(),
            TokenKind::Or => ParseRule::none(),
            TokenKind::Return => ParseRule::none(),
            TokenKind::Super => ParseRule::none(),
            TokenKind::This => ParseRule::none(),
            TokenKind::True => ParseRule::none(),
            TokenKind::Var => ParseRule::none(),
            TokenKind::While => ParseRule::none(),
            TokenKind::Print => ParseRule::none(),
            TokenKind::Eof => ParseRule::none(),
            TokenKind::Error => ParseRule::none(),
        }
    }

    fn consume(&mut self, token_kind: TokenKind, error_msg: &str) {
        if let Some(token) = self.current.as_ref() {
            if token.kind == token_kind {
                self.advance();
                return;
            }
        }
        self.error_at_current("");
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after expression.")
    }

    fn emit_byte(&mut self, byte: u8) {
        self.current_chunk
            .write(byte, self.previous.as_ref().unwrap().line)
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2)
    }

    fn emit_return(&mut self) {
        self.emit_byte(Op::Return.u8())
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        if !self.had_error {
            self.current_chunk.disassemble("code")
        }
    }

    fn emit_constant(&mut self, val: Value) {
        let konst = self.make_constant(val);
        self.emit_bytes(Op::Constant.u8(), konst)
    }

    fn make_constant(&mut self, val: Value) -> u8 {
        let constant_idx = self.current_chunk.add_constant(val);
        constant_idx
            .try_into()
            .expect("too many constants in one chunk")
    }

    fn error(&mut self, message: &str) {
        self.error_at(self.previous.clone(), message)
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.current.clone(), message);
    }

    fn error_at(&mut self, token: Option<Token>, message: &str) {
        if self.panic_mode {
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
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
enum Precedence {
    None = 0,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

type ParseFn = fn(&mut Parser) -> ();

struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: Precedence,
}

impl ParseRule {
    pub fn new(prefix: Option<ParseFn>, infix: Option<ParseFn>, precedence: Precedence) -> Self {
        Self {
            prefix,
            precedence,
            infix,
        }
    }

    pub fn none() -> Self {
        Self {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }
    }
}

impl From<u8> for Precedence {
    fn from(byte: u8) -> Self {
        unsafe { core::mem::transmute(byte) }
    }
}
