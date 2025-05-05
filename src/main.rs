mod ast;
mod chunk;
mod compiler;
mod core;
mod opcode;
mod parser;
mod scanner;
mod vm;

use core::value::Value;
use std::{
    env::args,
    fs::File,
    io::{self, Read, Write},
    process::exit,
    time::Instant,
};

use chunk::Chunk;
use compiler::Compiler;
use opcode::OpCode;
use parser::Parser;
use scanner::Scanner;
use thiserror::Error;
use vm::VM;

#[derive(Debug, Error)]
pub enum InterpretError {
    #[error("Compile error")]
    CompileError,
    #[error("Runtime error")]
    RuntimeError,
}

fn interpret(source: &str) {
    let scanner = Scanner::new(source);
    let parser = Parser::new(scanner);

    let mut vm = VM::new();
    let mut compiler = Compiler::new(parser, &mut vm);
    let chunk = compiler.compile().unwrap();

    vm.run(&chunk).unwrap();
}

fn repl() {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");

        interpret(&line);
    }
}

fn run_file(path: &str) {
    let mut file = File::open(path).expect("Failed to open file");

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read file");

    interpret(&contents);
}

fn main() {
    let args: Vec<_> = args().collect();
    if args.len() == 1 {
        repl();
    } else if args.len() == 2 {
        let start = Instant::now();
        run_file(&args[1]);
        eprintln!("Took {:?}", start.elapsed())
    } else {
        eprintln!("Usage: {} [script]", args[0]);
        exit(64);
    }
}
