use crate::opcodes::Op;
use std::usize;
pub type Value = f64;
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    lines: Vec<usize>,
}

impl Chunk {
    pub fn init() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }
    pub fn write(&mut self, byte: u8, line: usize) {
        self.lines.push(line);
        self.code.push(byte);
    }

    pub fn disassemble(&mut self, name: &str) {
        println!("== {} ==", name);
        let mut offset = 0;
        loop {
            if offset >= self.code.len() {
                break;
            }
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn write_constant(&mut self, value: Value, line: usize) {
        self.lines.push(line);
        let constant = self.add_constant(value);
        if constant < 256 {
            self.write(Op::Constant.u8(), line);
            self.write(constant as u8, line);
        } else if constant < 16_777_216 {
            self.write(Op::ConstantLong.u8(), line);
            let byte_representation = constant.to_le_bytes();
            let (operand, _) = byte_representation.split_at(3);
            operand.iter().for_each(|b| self.write(*b, line));
        } else {
            panic!("Max alox constant reached! (16.7m constants)")
        }
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);

        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("    | ");
        } else {
            print!("  {} ", self.lines[offset]);
        }

        let instruction = self.code[offset];
        let opcode = Op::from_u8(instruction);

        match opcode {
            Op::Constant => self.print_constant_instruction(opcode, offset),
            Op::ConstantLong => self.print_constant_long_instruction(opcode, offset),
            _default => {
                println!("{:?}", opcode);
                return offset + 1;
            }
        }
    }

    fn print_constant_instruction(&self, op: Op, offset: usize) -> usize {
        let constant = self.code[offset + 1];
        let value = self.constants[constant as usize];
        println!("{:?} \t{} '{}'", op, offset, value);
        offset + 2
    }

    fn print_constant_long_instruction(&self, op: Op, offset: usize) -> usize {
        let start = offset + 1;
        let end = offset + 3;
        let mut index = [0u8; 4];
        let constant = &self.code[start..=end];
        let (num, padding) = index.split_at_mut(constant.len());
        num.copy_from_slice(constant);
        padding.fill(0);
        let constant = u32::from_le_bytes(index);
        let value = self.constants[constant as usize];

        println!("{:?} \t{} '{}'", op, offset, value);
        offset + 4
    }
}
