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

            let mut has_error = false;
            for (i, line) in file_contents.lines().enumerate() {
                let mut chars = line.chars().peekable();
                while let Some(c) = chars.next() {
                    let token = match c {
                        '(' => {
                            Token::create(TokenType::LeftParen, "LEFT_PAREN", c.to_string(), i + 1)
                        }
                        ')' => Token::create(
                            TokenType::RightParen,
                            "RIGHT_PAREN",
                            c.to_string(),
                            i + 1,
                        ),
                        '{' => {
                            Token::create(TokenType::LeftBrace, "LEFT_BRACE", c.to_string(), i + 1)
                        }
                        '}' => Token::create(
                            TokenType::RightBrace,
                            "RIGHT_BRACE",
                            c.to_string(),
                            i + 1,
                        ),
                        '*' => Token::create(TokenType::Star, "STAR", c.to_string(), i + 1),
                        '/' => Token::create(TokenType::Slash, "SLASH", c.to_string(), i + 1),
                        ';' => {
                            Token::create(TokenType::Semicolon, "SEMICOLON", c.to_string(), i + 1)
                        }
                        '+' => Token::create(TokenType::Plus, "PLUS", c.to_string(), i + 1),
                        '-' => Token::create(TokenType::Minus, "MINUS", c.to_string(), i + 1),
                        '.' => Token::create(TokenType::Dot, "DOT", c.to_string(), i + 1),
                        ',' => Token::create(TokenType::Comma, "COMMA", c.to_string(), i + 1),
                        '=' => {
                            if chars.peek() == Some(&'=') {
                                chars.next();
                                Token::create(
                                    TokenType::EqualEqual,
                                    "EQUAL_EQUAL",
                                    "==".to_string(),
                                    i + 1,
                                )
                            } else {
                                Token::create(TokenType::Equal, "EQUAL", c.to_string(), i + 1)
                            }
                        }
                        '!' => {
                            if chars.peek() == Some(&'=') {
                                chars.next();
                                Token::create(
                                    TokenType::BangEqual,
                                    "BANG_EQUAL",
                                    "!=".to_string(),
                                    i + 1,
                                )
                            } else {
                                Token::create(TokenType::Bang, "BANG", c.to_string(), i + 1)
                            }
                        }
                        // '<' => {
                        //     if chars.peek() == Some(&'=') {
                        //         chars.next();
                        //         Token::create(
                        //             TokenType::LessEqual,
                        //             "LESS_EQUAL",
                        //             "<=".to_string(),
                        //             i+1,
                        //         )
                        //     } else {
                        //         Token::create(TokenType::LessThan, "LESS", c.to_string(), i+1)
                        //     }
                        // }
                        // '>' => {
                        //     if chars.peek() == Some(&'=') {
                        //         chars.next();
                        //         Token::create(
                        //             TokenType::GreaterEqual,
                        //             "GREATER_EQUAL",
                        //             ">=".to_string(),
                        //             i+1,
                        //         )
                        //     } else {
                        //         Token::create(TokenType::GreaterThan, "GREATER", c.to_string(), i+1)
                        //     }
                        // }
                        _ => Token::create(TokenType::Error, "ERROR", c.to_string(), i + 1),
                    };

                    if token.token_type == TokenType::Error {
                        has_error = true;
                        writeln!(io::stderr(), "{}", token).unwrap()
                    } else {
                        println!("{}", token)
                    }
                }
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
