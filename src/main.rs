use std::{
    env::args,
    fs::File,
    io::{self, Read, Write},
    process::exit,
    time::Instant,
};

use lox_bytecode_vm::interpret;
use lox_bytecode_vm::vm::VM;

fn repl() {
    let mut vm = VM::new(Box::new(std::io::stdout()));
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");

        interpret(&line, &mut vm, io::stderr());
    }
}

fn run_file(path: &str) {
    let mut file = File::open(path).expect("Failed to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read file");

    let mut vm = VM::new(Box::new(std::io::stdout()));
    interpret(&contents, &mut vm, io::stderr());
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
