mod ast;
mod chunk;
mod compiler;
mod core;
mod functions;
mod heap;
mod opcode;
mod parser;
mod scanner;
pub mod vm;

use std::io::Write;

use compiler::Compiler;
use parser::Parser;
use scanner::Scanner;
use vm::{Frame, VM};

pub fn interpret(source: &str, vm: &mut VM, mut err_writer: impl Write) {
    let scanner = Scanner::new(source);
    let parser = Parser::new(scanner);

    let main = Compiler::new(parser, vm.heap_mut()).compile();
    match main {
        Ok(main) => {
            let frame = Frame::new(main, 0);
            if let Err(e) = vm.run(frame) {
                writeln!(err_writer, "{e}").unwrap();
            }
        }
        Err(errs) => errs
            .iter()
            .for_each(|e| writeln!(err_writer, "{e}").unwrap()),
    }
}
