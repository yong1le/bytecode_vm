mod ast;
mod bytecode;
mod core;
mod frontend;
mod object;
mod runtime;

use std::io::Write;
use std::rc::Rc;

use bytecode::Compiler;
use frontend::Parser;
use frontend::Scanner;
use object::Closure;
use runtime::Frame;

pub use runtime::VM;

pub fn interpret(source: &str, vm: &mut VM, mut err_writer: impl Write) {
    let scanner = Scanner::new(source);
    let parser = Parser::new(scanner);

    let main = Compiler::new(parser, vm.heap_mut()).compile();
    match main {
        Ok(main) => {
            let frame = Frame::new(Rc::new(Closure::new(Rc::new(main), 0)), 0);
            if let Err(e) = vm.run(frame) {
                writeln!(err_writer, "{e}").unwrap();
            }
        }
        Err(errs) => errs
            .iter()
            .for_each(|e| writeln!(err_writer, "{e}").unwrap()),
    }
}
