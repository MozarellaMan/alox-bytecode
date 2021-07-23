use std::convert::{TryFrom, TryInto};


#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Op {
    Return = 0,
    Constant,
    ConstantLong,
    Add,
    Subract,
    Multiply,
    Divide,
    Negate
}

impl Op {
    pub fn u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(byte: u8) -> Self {
        byte.try_into().expect("unexpected opcode!")
    }
}

impl TryFrom<u8> for Op {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > Op::Negate as u8 {
            Err(())
        } else {
            unsafe { Ok(core::mem::transmute(value)) }
        }
    }
}