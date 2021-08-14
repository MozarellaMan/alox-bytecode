use chunk::Chunk;
use interner::Interner;
use parser::Parser;
use scanner::Scanner;
use typed_arena::Arena;
use vm::Vm;

pub mod chunk;
pub mod compiler;
pub mod interner;
pub mod object;
pub mod opcodes;
pub mod parser;
pub mod repl;
pub mod scanner;
pub mod token;
pub mod value;
pub mod vm;

pub fn run_script(source: &str) {
    let arena = Arena::new();
    let mut interner = Interner::new(&arena);
    let mut chunk = Chunk::init();

    let comp_result = {
        let scanner = Scanner::new(source);
        let mut parser = Parser::new(scanner, &mut chunk, &mut interner);
        parser.compile()
    };

    if comp_result.is_ok() {
        let mut vm = Vm::new(chunk, interner);

        if let Err(err) = vm.run() {
            eprintln!("{}", err)
        };
    }
}
