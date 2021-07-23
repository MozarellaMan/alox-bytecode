use std::fs;

use alox_bytecode::{chunk::Chunk, opcodes::Op, repl::run_prompt, vm::Vm};
use clap::{App, Arg, SubCommand};

fn main() {
    let mut chunk = Chunk::init();
    chunk.write_constant(1.2, 123);
    chunk.write_constant(3.4, 123);
    chunk.write(Op::Add.u8(), 123);
    chunk.write_constant(5.6, 123);
    chunk.write(Op::Divide.u8(), 123);
    chunk.write(Op::Negate.u8(), 123);
    chunk.write(Op::Return.u8(), 123);

    let mut vm = Vm::new(chunk);
    if let Err(err) = vm.interpret_current_chunk() {
        eprintln!("{:?}", err);
    }

    //chunk.disassemble("test chunk")

    const VERSION: &str = env!("CARGO_PKG_VERSION");
    let matches = App::new("Alox Byrecode")
        .version(VERSION)
        .author("Ayomide B. <ayoeze@hotmail.com>")
        .about("A Lox programming language")
        .arg(
            Arg::with_name("script")
                .short("s")
                .long("script")
                .value_name("FILE")
                .takes_value(true)
                .help("Sets an input script file to run"),
        )
        .subcommand(SubCommand::with_name("repl").about("a REPL"))
        .get_matches();

    if let Some("repl") = matches.subcommand_name() {
        run_prompt()
    }
    if let Some(filepath) = matches.value_of("script") {
        let file =  fs::read_to_string(filepath);
        match file {
            Ok(contents) => alox_bytecode::run_script(&contents),
            Err(err) => println!("Can't open file: {:?}", err),
        }
    } else {
        run_prompt()
    }
}
