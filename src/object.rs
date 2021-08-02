use std::fmt::Display;

use crate::interner::Interner;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    String(AloxString),
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct AloxString(pub u32);

impl Object {
    pub fn from_str(contents: &str, interner: &mut Interner) -> Self {
        Self::String(AloxString(interner.intern(contents)))
    }

    pub fn from_string(string: String, interner: &mut Interner) -> Self {
        Self::String(AloxString(interner.intern(&string)))
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::String(s) => write!(f, "{}", s.0),
        }
    }
}
