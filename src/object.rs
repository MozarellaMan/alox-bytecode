use std::fmt::Display;

use crate::value::AloxString;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    String(AloxString),
}

impl Object {
    pub fn from_str(contents: &str) -> Self {
        Self::String(AloxString(String::from(contents)))
    }

    pub fn from_string(string: String) -> Self {
        Self::String(AloxString(string))
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::String(s) => write!(f, "{}", s.0),
        }
    }
}
