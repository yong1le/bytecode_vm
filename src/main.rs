mod interpreter;
mod parser;
mod scanner;
mod token;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::exit;

use interpreter::EvalError;
use interpreter::Interpreter;
use parser::stmt::Stmt;
use parser::Parser;
use scanner::Scanner;
use token::Literal;

fn read_file(filename: &String) -> String {
    fs::read_to_string(filename).unwrap_or_else(|_| {
        writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
        String::new()
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        writeln!(io::stderr(), "Usage: {} tokenize <filename>", args[0]).unwrap();
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
                        writeln!(io::stderr(), "{e}");
                    }
                };
            }
            println!("EOF  null");

            exit(exit_code);
        }
        "parse" => {
            let scanner = Scanner::new(&file_contents);
            let parser = Parser::new(scanner);

            for expr in parser {
                match expr {
                    Ok(expr) => {
                        println!("{expr}");
                    }
                    Err(e) => {
                        exit_code = 65;
                        writeln!(io::stderr(), "{e}");
                    }
                }
            }

            exit(exit_code);
        }
        "evaluate" => {
            let scanner = Scanner::new(&file_contents);
            let parser = Parser::new(scanner);
            let mut interpreter = Interpreter::new();

            for stmt in parser {
                match stmt {
                    Ok(Stmt::Print(expr)) | Ok(Stmt::Expr(expr)) => {
                        match interpreter.evaluate(&expr) {
                            Ok(val) => match val {
                                Literal::Number(n) => {
                                    println!("{n}")
                                }
                                _ => println!("{val}"),
                            },
                            Err(e) => {
                                exit_code = 70;
                                writeln!(io::stderr(), "{e}");
                            }
                        }
                    }
                    Err(e) => {
                        exit_code = 65;
                        writeln!(io::stderr(), "{e}");
                    }
                }
            }

            exit(exit_code);
        }
        "run" => {
            let scanner = Scanner::new(&file_contents);
            let parser = Parser::new(scanner);
            let mut interpreter = Interpreter::new();

            for stmt in parser {
                match stmt {
                    Ok(stmt) => interpreter.interpret(&stmt),
                    Err(e) => {
                        exit_code = 65;
                        writeln!(io::stderr(), "{}", e);
                    }
                }
            }
        }
        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }
}
