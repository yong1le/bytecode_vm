mod scanner;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::exit;

use scanner::Token;
use scanner::TokenType;

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

    match command.as_str() {
        "tokenize" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
                String::new()
            });

            let mut i = 1;
            let mut has_error = false;
            for line in file_contents.lines() {
                for c in line.chars() {
                    let (token_type, token_str) = match c {
                        '(' => (TokenType::LeftParen, "LEFT_PAREN"),
                        ')' => (TokenType::RightParen, "RIGHT_PAREN"),
                        '{' => (TokenType::LeftBrace, "LEFT_BRACE"),
                        '}' => (TokenType::RightBrace, "RIGHT_BRACE"),
                        '*' => (TokenType::Star, "STAR"),
                        '/' => (TokenType::Slash, "SLASH"),
                        ';' => (TokenType::Semicolon, "SEMICOLON"),
                        '+' => (TokenType::Plus, "PLUS"),
                        '-' => (TokenType::Minus, "MINUS"),
                        '.' => (TokenType::Dot, "DOT"),
                        ',' => (TokenType::Comma, "COMMA"),
                        _ => (TokenType::Error, "ERROR"),
                    };

                    let token = Token {
                        token_type,
                        token_str: token_str.to_string(),
                        lexeme: c.to_string(),
                        line: i,
                    };

                    if token.token_type == TokenType::Error {
                        has_error = true;
                        writeln!(io::stderr(), "{}", token).unwrap()
                    } else {
                        println!("{}", token)
                    }
                }
                i+=1;
            }
            println!("EOF  null");

            if has_error {
               exit(65);
            }
        }

        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }
}
