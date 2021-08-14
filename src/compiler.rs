use crate::token::Token;

pub const U8_COUNT: usize = (u8::MAX as usize) + 1;

pub struct Compiler<'a> {
    pub locals: [Local<'a>; U8_COUNT],
    pub count: usize,
    pub scope_depth: i32,
}

#[derive(Clone, Default, Copy, Debug)]
pub struct Local<'a> {
    pub name: Token<'a>,
    pub depth: i32,
}

impl Compiler<'_> {
    pub fn new() -> Self {
        let locals = [Local::default(); U8_COUNT];
        Self {
            count: 0,
            scope_depth: 0,
            locals,
        }
    }

    #[inline]
    pub fn increase_scope(&mut self) {
        self.scope_depth += 1;
    }

    #[inline]
    pub fn decrease_scope(&mut self) {
        self.scope_depth -= 1;
    }
}
