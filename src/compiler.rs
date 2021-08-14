use crate::token::Token;

const U8_COUNT: usize = (u8::MAX as usize) + 1;

pub struct Compiler<'a> {
    locals: [Local<'a>; U8_COUNT],
    count: usize,
    scope_depth: usize,
}

#[derive(Clone, Default, Copy)]
pub struct Local<'a> {
    name: Token<'a>,
    depth: usize,
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
}
