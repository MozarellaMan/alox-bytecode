use chunk::Chunk;
use vm::Vm;

pub mod chunk;
pub mod compiler;
pub mod opcodes;
pub mod repl;
pub mod scanner;
pub mod token;
pub mod value;
pub mod vm;

pub fn run_script(source: &str) {
    let mut vm = Vm::new(Chunk::init());
    if let Err(err) = vm.interpret(source) {
        eprintln!("{:?}", err)
    };
}
