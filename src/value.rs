use std::fmt::Display;

use crate::object::Object;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Obj(Object),
    Bool(bool),
    Number(f64),
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AloxString(pub String);

impl Value {
    pub fn from_str(contents: &str) -> Self {
        Self::Obj(Object::from_str(contents))
    }

    pub fn from_string(string: String) -> Self {
        Self::Obj(Object::from_string(string))
    }

    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Self::Bool(bool) => Some(bool),
            _ => None,
        }
    }
    pub fn as_number(&self) -> Option<f64> {
        match *self {
            Self::Number(num) => Some(num),
            _ => None,
        }
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
