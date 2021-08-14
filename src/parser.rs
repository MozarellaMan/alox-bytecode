use std::{convert::TryInto, u8};

use crate::{
    chunk::Chunk,
    compiler::Compiler,
    interner::Interner,
    opcodes::Op,
    scanner::Scanner,
    token::{Token, TokenKind},
    value::Value,
};

pub type CompilationResult = Result<(), CompilationError>;
pub struct Parser<'source, 'chunk, 'interner> {
    scanner: Scanner<'source>,
    current: Option<Token<'source>>,
    previous: Option<Token<'source>>,
    current_chunk: &'chunk mut Chunk,
    interner: &'chunk mut Interner<'interner>,
    had_error: bool,
    panic_mode: bool,
    current_compiler: Compiler<'source>,
}

impl<'source, 'chunk, 'interner> Parser<'source, 'chunk, 'interner> {
    pub fn new(
        scanner: Scanner<'source>,
        chunk: &'chunk mut Chunk,
        interner: &'chunk mut Interner<'interner>,
    ) -> Self {
        Self {
            scanner,
            current: None,
            previous: None,
            had_error: false,
            panic_mode: false,
            current_chunk: chunk,
            current_compiler: Compiler::new(),
            interner,
        }
    }

    pub fn compile(&mut self) -> CompilationResult {
        self.advance();
        while !self.match_current(TokenKind::Eof) {
            self.declaration();
        }
        if self.had_error {
            Err(CompilationError::Error)
        } else {
            self.end_compiler();
            Ok(())
        }
    }

    fn match_current(&mut self, kind: TokenKind) -> bool {
        if !self.check(kind) {
            false
        } else {
            self.advance();
            true
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.current.expect("no current token to check!").kind == kind
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

    fn declaration(&mut self) {
        if self.match_current(TokenKind::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }
        if self.panic_mode {
            self.synchronize();
        }
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.match_current(TokenKind::Equal) {
            self.expression();
        } else {
            self.emit_byte(Op::Nil.u8())
        }

        self.consume(
            TokenKind::Semicolon,
            "Expect ';' after variable declaration.",
        );

        self.define_variable(global);
    }

    fn statement(&mut self) {
        if self.match_current(TokenKind::Print) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenKind::Semicolon, "Expected ';' after expression.");
        self.emit_byte(Op::Pop.u8());
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenKind::Semicolon, "Expected ';' after value.");
        self.emit_byte(Op::Print.u8())
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self, _can_assign: bool) {
        let value = self.previous_token().lexeme.parse::<f64>().unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn unary(&mut self, _can_assign: bool) {
        let op_kind = self.previous_token().kind;

        // compile operand
        self.parse_precedence(Precedence::Unary);

        // emit op instruction
        match op_kind {
            TokenKind::Minus => self.emit_byte(Op::Negate.u8()),
            TokenKind::Bang => self.emit_byte(Op::Not.u8()),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self, _can_assign: bool) {
        let op_kind = self.previous_token().kind;
        let rule = self.find_rule(op_kind);
        self.parse_precedence((rule.precedence as u8 + 1).into());

        match op_kind {
            TokenKind::Plus => self.emit_byte(Op::Add.u8()),
            TokenKind::Minus => self.emit_byte(Op::Subtract.u8()),
            TokenKind::Star => self.emit_byte(Op::Multiply.u8()),
            TokenKind::Slash => self.emit_byte(Op::Divide.u8()),
            TokenKind::BangEqual => self.emit_bytes(Op::Equal.u8(), Op::Not.u8()),
            TokenKind::EqualEqual => self.emit_byte(Op::Equal.u8()),
            TokenKind::Greater => self.emit_byte(Op::Greater.u8()),
            TokenKind::GreaterEqual => self.emit_bytes(Op::Less.u8(), Op::Not.u8()),
            TokenKind::Less => self.emit_byte(Op::Less.u8()),
            TokenKind::LessEqual => self.emit_bytes(Op::Greater.u8(), Op::Not.u8()),
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self.find_rule(self.previous_token().kind).prefix;
        let can_assign = precedence as u8 <= Precedence::Assignment as u8;

        if let Some(rule) = prefix_rule {
            rule(self, can_assign);
        } else {
            self.error("Expected expression.");
            return;
        }

        while precedence as u8 <= self.find_rule(self.current_token().kind).precedence as u8 {
            self.advance();
            let infix_rule = self.find_rule(self.previous_token().kind).infix;
            if let Some(infix) = infix_rule {
                infix(self, can_assign)
            }
        }

        if can_assign && self.match_current(TokenKind::Equal) {
            self.error("Invalid assignment target.")
        }
    }

    fn parse_variable(&mut self, error_msg: &str) -> u8 {
        self.consume(TokenKind::Identifier, error_msg);
        let name = self.previous.expect("No previous token!").lexeme;
        self.identifier_constant(name)
    }

    fn identifier_constant(&mut self, name: &str) -> u8 {
        let idx = self.interner.intern(name);
        self.make_constant(Value::from_str_index(idx))
    }

    fn define_variable(&mut self, global: u8) {
        self.emit_bytes(Op::DefineGlobal.u8(), global)
    }

    fn find_rule(&mut self, op_kind: TokenKind) -> ParseRule {
        match op_kind {
            TokenKind::LeftParen => {
                ParseRule::new(Some(|this, b| this.grouping(b)), None, Precedence::None)
            }
            TokenKind::Minus => ParseRule::new(
                Some(|this, b| this.unary(b)),
                Some(|this, b| this.binary(b)),
                Precedence::Term,
            ),
            TokenKind::Plus => {
                ParseRule::new(None, Some(|this, b| this.binary(b)), Precedence::Term)
            }
            TokenKind::Slash => {
                ParseRule::new(None, Some(|this, b| this.binary(b)), Precedence::Factor)
            }
            TokenKind::Star => {
                ParseRule::new(None, Some(|this, b| this.binary(b)), Precedence::Factor)
            }
            TokenKind::Bang => {
                ParseRule::new(Some(|this, b| this.unary(b)), None, Precedence::None)
            }
            TokenKind::BangEqual => {
                ParseRule::new(None, Some(|this, b| this.binary(b)), Precedence::Equality)
            }
            TokenKind::EqualEqual => {
                ParseRule::new(None, Some(|this, b| this.binary(b)), Precedence::Equality)
            }
            TokenKind::Greater => {
                ParseRule::new(None, Some(|this, b| this.binary(b)), Precedence::Comparison)
            }
            TokenKind::GreaterEqual => {
                ParseRule::new(None, Some(|this, b| this.binary(b)), Precedence::Comparison)
            }
            TokenKind::Less => {
                ParseRule::new(None, Some(|this, b| this.binary(b)), Precedence::Comparison)
            }
            TokenKind::LessEqual => {
                ParseRule::new(None, Some(|this, b| this.binary(b)), Precedence::Comparison)
            }
            TokenKind::Identifier => {
                ParseRule::new(Some(|this, b| this.variable(b)), None, Precedence::None)
            }
            TokenKind::String => {
                ParseRule::new(Some(|this, b| this.string(b)), None, Precedence::None)
            }
            TokenKind::Number => {
                ParseRule::new(Some(|this, b| this.number(b)), None, Precedence::None)
            }
            TokenKind::False => {
                ParseRule::new(Some(|this, b| this.literal(b)), None, Precedence::None)
            }
            TokenKind::Nil => {
                ParseRule::new(Some(|this, b| this.literal(b)), None, Precedence::None)
            }
            TokenKind::True => {
                ParseRule::new(Some(|this, b| this.literal(b)), None, Precedence::None)
            }
            TokenKind::RightParen
            | TokenKind::LeftBrace
            | TokenKind::RightBrace
            | TokenKind::Comma
            | TokenKind::Dot
            | TokenKind::Semicolon
            | TokenKind::Equal
            | TokenKind::Var
            | TokenKind::While
            | TokenKind::Print
            | TokenKind::Eof
            | TokenKind::Error
            | TokenKind::And
            | TokenKind::Class
            | TokenKind::Else
            | TokenKind::Fun
            | TokenKind::For
            | TokenKind::If
            | TokenKind::Or
            | TokenKind::Return
            | TokenKind::Super
            | TokenKind::This => ParseRule::none(),
        }
    }

    fn variable(&mut self, can_assign: bool) {
        let previous = self.previous.expect("No previous token!").lexeme;
        self.named_variable(previous, can_assign);
    }

    fn named_variable(&mut self, name: &str, can_assign: bool) {
        let arg = self.identifier_constant(name);
        if can_assign && self.match_current(TokenKind::Equal) {
            self.expression();
            self.emit_bytes(Op::SetGlobal.u8(), arg);
        } else {
            self.emit_bytes(Op::GetGlobal.u8(), arg);
        }
    }

    fn literal(&mut self, _can_assign: bool) {
        match self.previous_token().kind {
            TokenKind::False => self.emit_byte(Op::False.u8()),
            TokenKind::True => self.emit_byte(Op::True.u8()),
            TokenKind::Nil => self.emit_byte(Op::Nil.u8()),
            _ => unreachable!(),
        }
    }

    fn string(&mut self, _can_assign: bool) {
        let string = {
            let string = self.previous_token();
            let string_len = string.lexeme.len();
            let string = &string.lexeme[1..string_len - 1];
            if self.interner.exists(string) {
                Ok(string)
            } else {
                Err(String::from(string))
            }
        };

        let val = match string {
            Ok(existing) => {
                let idx = self.interner.get_existing(existing);
                Value::from_str_index(idx)
            }
            Err(new_string) => {
                let idx = self.interner.intern(&new_string);
                Value::from_str_index(idx)
            }
        };
        self.emit_constant(val);
    }

    fn consume(&mut self, token_kind: TokenKind, error_msg: &str) {
        if let Some(token) = self.current.as_ref() {
            if token.kind == token_kind {
                self.advance();
                return;
            }
        }
        self.error_at_current(error_msg);
    }

    fn grouping(&mut self, _can_assign: bool) {
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
            self.current_chunk.disassemble("code", self.interner)
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

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while !self.check(TokenKind::Eof) {
            if self.previous_token().kind == TokenKind::Semicolon {
                return;
            }

            if let Some(tok) = self.current {
                match tok.kind {
                    TokenKind::Class
                    | TokenKind::Fun
                    | TokenKind::Var
                    | TokenKind::For
                    | TokenKind::If
                    | TokenKind::While
                    | TokenKind::Print
                    | TokenKind::Return => {
                        return;
                    }
                    _ => {}
                }
            } else {
                return;
            }

            self.advance();
        }
    }

    fn error(&mut self, message: &str) {
        self.error_at(self.previous, message)
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.current, message);
    }

    fn error_at(&mut self, token: Option<Token>, message: &str) {
        self.had_error = true;
        if self.panic_mode {
            return;
        }
        if let Some(token) = token {
            eprint!("[line {}] Error", token.line);
            match token.kind {
                TokenKind::Eof => eprint!(" at end"),
                TokenKind::Error => {}
                _ => eprint!(" at '{}' ", token.lexeme),
            }
            if !message.is_empty() {
                eprintln!(": {}", message);
            } else {
                eprint!("\n");
            }
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

type ParseFn = fn(&mut Parser, bool) -> ();

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
