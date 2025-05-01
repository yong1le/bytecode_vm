mod ast;
mod core;
mod parser;
mod resolver;
mod runtime;
mod scanner;

use itertools::{Either, Itertools};
use std::env;
use std::fs;
use std::process::exit;

use interpreter::Interpreter;
use parser::Parser;
use resolver::Resolver;
use runtime::interpreter;
use scanner::Scanner;

fn read_file(filename: &String) -> String {
    fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Failed to read file {}", filename);
        String::new()
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Usage: {} [tokenize|parse|evaluate|run] <filename>",
            args[0]
        )
    }

    let command = &args[1];
    let filename = &args[2];

    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

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
                        eprintln!("{e}");
                    }
                };
            }

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
                        eprintln!("{e}");
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
                            eprintln!("{e}");
                        }
                    },
                    Err(e) => {
                        exit_code = 65;
                        eprintln!("{e}");
                    }
                }
            }

            exit(exit_code);
        }
        "run" => {
            let scanner = Scanner::new(&file_contents);
            let parser = Parser::new(scanner);

            let (statements, errs): (Vec<_>, Vec<_>) = parser.partition_map(|r| match r {
                Ok(v) => Either::Left(v),
                Err(v) => Either::Right(v),
            });

            // Return early if any syntax errors
            errs.iter().for_each(|e| {
                eprintln!("{e}");
            });
            if !errs.is_empty() {
                exit(65);
            }

            let mut interpreter = Interpreter::new();
            let mut resolver = Resolver::new(&mut interpreter);

            // First pass, resolve variable bindings
            statements
                .iter()
                .for_each(|stmt| match resolver.resolve_stmt(stmt) {
                    Ok(()) => (),
                    Err(e) => {
                        eprintln!("{e}");
                        exit_code = 65;
                    }
                });

            if exit_code != 0 {
                exit(exit_code);
            }

            // Second pass, interpret statements
            statements
                .iter()
                .for_each(|stmt| match interpreter.interpret(stmt) {
                    Ok(()) => (),
                    Err(e) => {
                        eprintln!("{e}");
                        exit(70);
                    }
                });

            exit(exit_code);
        }
        _ => {
            eprintln!("Unknown command: {}", command)
        }
    }
}
