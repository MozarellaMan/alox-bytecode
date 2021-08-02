use crate::{chunk::Chunk, interner::Interner, object::Object, opcodes::Op, value::Value};

const STACK_UNDERFLOW: &str = "Stack underflow!";

macro_rules! binary_op {
    ($self:ident,$operator:tt, $variant:tt) => {
        {
            let b = $self.pop();
            let a = $self.pop();
            if let (Value::Number(n1), Value::Number(n2)) = (&a, &b) {
                $self.push(Value::$variant(n1 $operator n2));
            } else {
                $self.push(a);
                $self.push(b);
                $self.runtime_error("Operands must be numbers.");
                return Err(InterpreterError::RuntimeError)
            }
        }
    };
}

pub type InterpreterResult = Result<(), InterpreterError>;
pub struct Vm<'a> {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    interner: Interner<'a>,
}

impl<'vm> Vm<'vm> {
    pub fn new(chunk: Chunk, interner: Interner<'vm>) -> Self {
        Vm {
            chunk,
            ip: 0,
            stack: Vec::new(),
            interner,
        }
    }

    pub fn interpret_current_chunk(&mut self) -> InterpreterResult {
        self.run()
    }

    pub fn run(&mut self) -> InterpreterResult {
        loop {
            if self.ip >= self.chunk.code.len() {
                break;
            }
            #[cfg(debug_assertions)]
            println!("{:?}", &self.stack);
            let next_byte = self.next_byte();
            let instruction = Op::from_u8(next_byte);
            #[cfg(debug_assertions)]
            self.chunk.disassemble_instruction(self.ip - 1);
            match instruction {
                Op::Return => match self.pop() {
                    Value::Obj(obj) => match obj {
                        Object::String(idx) => println!("{}", self.interner.lookup(idx.0)),
                    },
                    _other => println!("{}", _other),
                },
                Op::Constant | Op::ConstantLong => {
                    let index = self.next_byte();
                    let constant = self.read_constant(index);
                    self.push(constant);
                }
                Op::Negate => {
                    let val = self.pop();
                    if let Value::Number(n) = val {
                        self.push(Value::Number(-n));
                    } else {
                        self.push(val);
                        self.runtime_error("Operand must be a number.");
                        return Err(InterpreterError::RuntimeError);
                    }
                }
                Op::Add => {
                    let b = self.pop();
                    let a = self.pop();
                    match (&b, &a) {
                        (Value::Obj(b), Value::Obj(a)) => {
                            if let (Object::String(a), Object::String(b)) = (b, a) {
                                let first = {
                                    let str = self.interner.lookup(b.0);
                                    String::from(str)
                                };
                                let second = self.interner.lookup(a.0);
                                let concatenated = first + second;
                                let concatenated = self.interner.intern(&concatenated);
                                self.push(Value::from_str_index(concatenated));
                            } else {
                                self.push(Value::Obj(a.clone()));
                                self.push(Value::Obj(b.clone()));
                                self.runtime_error("Operands must be two strings.");
                                return Err(InterpreterError::RuntimeError);
                            }
                        }
                        (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a + b)),
                        _ => {
                            self.push(a);
                            self.push(b);
                            self.runtime_error("Operands must be two numbers.");
                            return Err(InterpreterError::RuntimeError);
                        }
                    }
                }
                Op::Subract => binary_op!(self, -, Number),
                Op::Multiply => binary_op!(self, *, Number),
                Op::Divide => binary_op!(self, /, Number),
                Op::Nil => self.push(Value::Nil),
                Op::True => self.push(Value::Bool(true)),
                Op::False => self.push(Value::Bool(false)),
                Op::Not => {
                    let val = self.pop();
                    self.push(Value::Bool(Vm::is_falsey(val)))
                }
                Op::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b))
                }
                Op::Greater => binary_op!(self, >, Bool),
                Op::Less => binary_op!(self, <, Bool),
            }
        }
        Ok(())
    }

    fn peek(&self) -> &Value {
        self.stack.last().expect(STACK_UNDERFLOW)
    }

    fn peek_mut(&mut self) -> &mut Value {
        self.stack.last_mut().expect(STACK_UNDERFLOW)
    }

    fn peek_by(&self, distance: usize) -> &Value {
        self.stack
            .get(self.stack.len() - 1 - distance)
            .expect(STACK_UNDERFLOW)
    }

    #[inline]
    fn pop(&mut self) -> Value {
        self.stack.pop().expect(STACK_UNDERFLOW)
    }

    #[inline]
    fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    fn next_byte(&mut self) -> u8 {
        let byte = self.chunk.code[self.ip];
        self.ip += 1;
        byte
    }

    fn read_constant(&self, index: u8) -> Value {
        self.chunk.constants[index as usize].clone()
    }

    fn runtime_error(&self, message: &str) {
        eprintln!("{}", message);
        let line = self.chunk.lines[self.ip - 1];
        eprintln!("[line {}] in script", line)
    }

    fn is_falsey(val: Value) -> bool {
        match val {
            Value::Nil => true,
            Value::Bool(b) => !b,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum InterpreterError {
    CompileError,
    RuntimeError,
    NoInstructions,
    UnknownInstruction,
}
