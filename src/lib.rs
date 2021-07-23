use chunk::Chunk;
use vm::Vm;

pub mod chunk;
pub mod opcodes;
pub mod vm;
pub mod repl;
pub mod compiler;
pub mod scanner;
pub mod token;

pub fn run_script(source: &str) {
    let mut vm = Vm::new(Chunk::init());
    if let Err(err) = vm.interpret(source) {
        eprintln!("{:?}", err)
    };
}