mod ast;
mod chunk;
mod compiler;
mod core;
mod heap;
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
use vm::VM;

fn interpret(source: &str, vm: &mut VM) {
    let scanner = Scanner::new(source);
    let parser = Parser::new(scanner);

    let chunk = Compiler::new(parser, vm.heap()).compile();

    match chunk {
        Ok(chunk) => {
            if let Err(e) = vm.run(chunk) {
                eprintln!("{e}")
            }
        }
        Err(e) => eprintln!("{e}"),
    }
}

fn repl() {
    let mut vm = VM::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");

        interpret(&line, &mut vm);
    }
}

fn run_file(path: &str) {
    let mut file = File::open(path).expect("Failed to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read file");

    let mut vm = VM::new();
    interpret(&contents, &mut vm);
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
