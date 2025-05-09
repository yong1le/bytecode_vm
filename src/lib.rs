use std::io::Write;

use compiler::Compiler;
use parser::Parser;
use scanner::Scanner;
use vm::VM;

mod ast;
mod chunk;
mod compiler;
mod core;
mod heap;
mod opcode;
mod parser;
mod scanner;
pub mod vm;

pub fn interpret(source: &str, vm: &mut VM, mut err_writer: impl Write) {
    let scanner = Scanner::new(source);
    let parser = Parser::new(scanner);

    let chunk = Compiler::new(parser, vm.heap()).compile();

    match chunk {
        Ok(chunk) => {
            if let Err(e) = vm.run(chunk) {
                writeln!(err_writer, "{e}").unwrap();
            }
        }
        Err(errs) => errs
            .iter()
            .for_each(|e| writeln!(err_writer, "{e}").unwrap()),
    }
}
