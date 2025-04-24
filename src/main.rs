mod ast;
mod errors;
mod interpreter;
mod parser;
mod scanner;
mod token;
mod environment;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::exit;

use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;

fn read_file(filename: &String) -> String {
    fs::read_to_string(filename).unwrap_or_else(|_| {
        writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
        String::new()
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        writeln!(
            io::stderr(),
            "Usage: {} [tokenize|parse|evaluate|run] <filename>",
            args[0]
        )
        .unwrap();
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    // You can use print statements as follows for debugging, they'll be visible when running tests.
    writeln!(io::stderr(), "Logs from your program will appear here!").unwrap();

    let file_contents = read_file(filename);

    let mut exit_code = 0;
    match command.as_str() {
        "tokenize" => {
            let scanner = Scanner::new(&file_contents);
            for token in scanner {
                match token {
                    Ok(t) => {
                        println!("{t}");
                    }
                    Err(e) => {
                        exit_code = 65;
                        writeln!(io::stderr(), "{e}").unwrap();
                    }
                };
            }
            println!("EOF  null");

            exit(exit_code);
        }
        "parse" => {
            let scanner = Scanner::new(&file_contents);
            let mut parser = Parser::new(scanner);

            if let Some(expr) = parser.parse() {
                match expr {
                    Ok(expr) => {
                        println!("{expr}");
                    }
                    Err(e) => {
                        exit_code = 65;
                        writeln!(io::stderr(), "{e}").unwrap();
                    }
                }
            }

            exit(exit_code);
        }
        "evaluate" => {
            let scanner = Scanner::new(&file_contents);
            let mut parser = Parser::new(scanner);
            let mut interpreter = Interpreter::new();

            if let Some(expr) = parser.parse() {
                match expr {
                    Ok(expr) => match interpreter.evaluate(&expr) {
                        Ok(val) => {
                            println!("{}", val.stringify());
                        }
                        Err(e) => {
                            exit_code = 70;
                            writeln!(io::stderr(), "{e}").unwrap();
                        }
                    },
                    Err(e) => {
                        exit_code = 65;
                        writeln!(io::stderr(), "{e}").unwrap();
                    }
                }
            }

            exit(exit_code);
        }
        "run" => {
            let scanner = Scanner::new(&file_contents);
            let parser = Parser::new(scanner);
            let mut interpreter = Interpreter::new();

            parser.for_each(|stmt| match stmt {
                Ok(s) => match interpreter.interpret(&s) {
                    Ok(()) => (),
                    Err(e) => {
                        writeln!(io::stderr(), "{e}").unwrap();
                        exit(70);
                    }
                },
                Err(e) => {
                    exit_code = 65;
                    writeln!(io::stderr(), "{e}").unwrap()
                }
            });

            exit(exit_code);
        }
        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }
}
