mod parser;
mod scanner;
mod token;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::exit;

use parser::Expr;
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

    match command.as_str() {
        "tokenize" => {
            let scanner = Scanner::new(&file_contents);
            let mut has_error = false;
            for token in scanner {
                match token {
                    Ok(t) => {
                        println!("{t}");
                    }
                    Err(e) => {
                        has_error = true;
                        writeln!(io::stderr(), "{e}");
                    }
                };
            }

            if has_error {
                exit(65);
            }
        }
        "parse" => {
            let scanner = Scanner::new(&file_contents);
            let parser = Parser::new(scanner);

            println!("{}", match parser.expression() {
                Expr::Literal(literal) => match literal {
                    Literal::Nil => "null".to_string(),
                    l => l.to_string()
                }
                c => c.to_string()
            });
        }
        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }
}
