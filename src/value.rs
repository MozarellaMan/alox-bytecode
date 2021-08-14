use std::fmt::Display;

use crate::{
    interner::Interner,
    object::{AloxString, Object},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Obj(Object),
    Bool(bool),
    Number(f64),
    Nil,
}

impl Value {
    pub fn from_str_index(idx: u32) -> Self {
        Self::Obj(Object::String(AloxString(idx)))
    }

    pub fn from_str(contents: &str, interner: &mut Interner) -> Self {
        Self::Obj(Object::from_str(contents, interner))
    }

    pub fn from_string(string: String, interner: &mut Interner) -> Self {
        Self::Obj(Object::from_str(&string, interner))
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(bool) = *self {
            Some(bool)
        } else {
            None
        }
    }
    pub fn as_number(&self) -> Option<f64> {
        if let Self::Number(num) = *self {
            Some(num)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<AloxString> {
        if let Self::Obj(obj) = &self {
            if let Object::String(string) = *obj {
                return Some(string);
            }
        }
        None
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(bool) => write!(f, "{}", bool),
            Value::Number(n) => write!(f, "{}", n),
            Value::Nil => write!(f, "Nil"),
            Value::Obj(obj) => write!(f, "{}", obj),
        }
    }
}
