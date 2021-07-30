use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Nil,
}

impl Value {
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
        }
    }
}
