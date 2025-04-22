mod parser;
mod scanner;
mod token;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::exit;
use std::string::ParseError;

use parser::Parser;
use scanner::ScanError;
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
            let mut parser = Parser::new(scanner);

            match parser.expression() {
                Ok(expr) => {
                    println!("{expr}");
                }
                Err(e) => {
                    exit_code = 65;
                    writeln!(io::stderr(), "{e}");
                }
            }

            exit(exit_code);
        }
        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }
}
