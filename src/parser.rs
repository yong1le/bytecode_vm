use std::{iter::Peekable, vec};

use crate::{
    ast::{expr::Expr, stmt::Stmt},
    errors::ParseError,
    scanner::Scanner,
    token::{Literal, Token, TokenType},
};

// program        →  declaration* EOF;
// declaration    -> varDecl | statement;
// varDecl        -> "var" IDENTIFIER ( "=" )? ";";
// statement      → exprStmt | printStmt | block ;
// block          -> "{" declaration* "}"
// exprStmt       → expression ";" '
// printStmt      → "print" expression ";" ;

// expression     → assignment ;
// assignment     -> IDENTIFIER "=" assignment | equality;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" | IDENTIFIER;

pub struct Parser<'a> {
    tokens: Peekable<Scanner<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Scanner<'a>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    /// Parses the first expression from the list of tokens and advances the
    /// iterator.
    pub fn parse(&mut self) -> Option<Result<Expr, ParseError>> {
        match &self.tokens.peek() {
            Some(Ok(token)) => match token.token {
                TokenType::Eof => return None,
                _ => (),
            },
            None => return None,
            _ => (),
        }

        let result = self.expression();

        Some(result)
    }

    /// Advances to the next token to parse. If there are no more tokens to parse,
    /// An `UnexpectedEOF` error is returned, because `advance()` is only called when
    /// the grammar expects another function
    fn advance(&mut self) -> Result<Token, ParseError> {
        match self.tokens.next() {
            Some(Ok(t)) => Ok(t),
            Some(Err(e)) => Err(ParseError::ScanError(e)),
            None => Err(ParseError::UnexpectedEOF),
        }
    }

    fn peek(&mut self) -> Result<&Token, ParseError> {
        match self.tokens.peek() {
            Some(Ok(t)) => Ok(t),
            Some(Err(e)) => Err(ParseError::ScanError(e.to_owned())),
            None => Err(ParseError::UnexpectedEOF),
        }
    }

    fn consume(&mut self, tokens: &Vec<TokenType>) -> Result<Token, ParseError> {
        let next_token = match self.tokens.peek() {
            Some(Ok(t)) => t,
            Some(Err(e)) => return Err(ParseError::ScanError(e.to_owned())),
            None => return Err(ParseError::UnexpectedEOF),
        };

        if tokens.contains(&next_token.token) {
            self.advance()
        } else {
            Err(ParseError::ExpectedChar(
                next_token.line,
                next_token.lexeme.to_owned(),
                format!("{:?}", tokens),
            ))
        }
    }

    fn synchronize(&mut self) {
        // Discard the value, since we know its going to be an error

        loop {
            let cur_token = match self.advance() {
                Ok(t) => t.token,
                Err(ParseError::UnexpectedEOF) => return,
                Err(_) => TokenType::Nil, // Anything that doesn't match below should work
            };

            if cur_token == TokenType::Semicolon {
                return;
            }

            let next_token = match self.peek() {
                Ok(t) => &t.token,
                Err(ParseError::UnexpectedEOF) => return,
                Err(_) => &TokenType::Nil,
            };

            match next_token {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => (),
            }
        }
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        let t = self.peek()?;

        match t.token {
            TokenType::Var => {
                self.advance()?;
                self.declare_var()
            }
            _ => self.statement(),
        }
    }

    fn declare_var(&mut self) -> Result<Stmt, ParseError> {
        let identifier_token = self.advance()?;

        match identifier_token.token {
            TokenType::Identifier => {
                if let Ok(_equals) = self.consume(&vec![TokenType::Equal]) {
                    let initializer = self.expression()?;
                    self.consume(&vec![TokenType::Semicolon])?;
                    Ok(Stmt::DeclareVar(identifier_token, initializer))
                } else {
                    self.consume(&vec![TokenType::Semicolon])?;
                    Ok(Stmt::DeclareVar(
                        identifier_token,
                        Expr::Literal(Literal::Nil),
                    ))
                }
            }
            _ => Err(ParseError::ExpectedChar(
                identifier_token.line,
                identifier_token.lexeme.to_string(),
                "IDENTIFIER".to_string(),
            )),
        }
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        let t = self.peek()?;

        match t.token {
            TokenType::Print => {
                self.advance()?;
                self.print_stmt()
            }
            TokenType::LeftBrace => {
                self.advance()?;
                self.block()
            }
            _ => self.expression_stmt(),
        }
    }

    fn print_stmt(&mut self) -> Result<Stmt, ParseError> {
        let print_expr = self.expression()?;
        self.consume(&vec![TokenType::Semicolon])?;
        Ok(Stmt::Print(print_expr))
    }

    fn block(&mut self) -> Result<Stmt, ParseError> {
        let mut statements = vec![];

        loop {
            let token = self.peek()?;
            match token.token {
                TokenType::Eof => {
                    return Err(ParseError::ExpectedChar(
                        token.line,
                        "EOF".to_string(),
                        format!("{}", TokenType::Semicolon)
                    ))
                }
                TokenType::RightBrace => break,
                _ => statements.push(self.declaration()?),
            }
        }

        self.consume(&vec![TokenType::RightBrace])?;
        Ok(Stmt::Block(statements))
    }

    /// Basically expression, but consume the semicolon
    fn expression_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(&vec![TokenType::Semicolon])?;
        Ok(Stmt::Expr(expr))
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.equality()?;

        let t = self.peek()?;

        match t.token {
            TokenType::Equal => {
                let actual = self.advance()?;
                let value = self.assignment()?;

                match expr {
                    Expr::Variable(id) => Ok(Expr::Assign(id, Box::new(value))),
                    a => Err(ParseError::InvalidAssignment(actual.line, a)),
                }
            }
            _ => Ok(expr),
        }
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        loop {
            let t = self.peek()?;

            match t.token {
                TokenType::EqualEqual | TokenType::BangEqual => {
                    let op = self.advance()?;
                    let right = self.comparison()?;
                    expr = Expr::Binary(op, Box::new(expr), Box::new(right))
                }
                _ => break,
            }
        }

        return Ok(expr);
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        loop {
            let t = self.peek()?;

            match t.token {
                TokenType::LessEqual
                | TokenType::LessThan
                | TokenType::GreaterEqual
                | TokenType::GreaterThan => {
                    let op = self.advance()?;
                    let right = self.term()?;
                    expr = Expr::Binary(op, Box::new(expr), Box::new(right))
                }
                _ => break,
            }
        }

        return Ok(expr);
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        loop {
            let t = self.peek()?;

            match t.token {
                TokenType::Plus | TokenType::Minus => {
                    let op = self.advance()?;
                    let right = self.factor()?;
                    expr = Expr::Binary(op, Box::new(expr), Box::new(right))
                }
                _ => break,
            }
        }

        return Ok(expr);
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        loop {
            let t = self.peek()?;

            match t.token {
                TokenType::Star | TokenType::Slash => {
                    let op = self.advance()?;
                    let right = self.unary()?;
                    expr = Expr::Binary(op, Box::new(expr), Box::new(right))
                }
                _ => break,
            }
        }

        return Ok(expr);
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        let t = self.peek()?;

        match t.token {
            TokenType::Bang | TokenType::Minus => {
                let op = self.advance()?;
                let expr = self.unary()?;
                Ok(Expr::Unary(op, Box::new(expr)))
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let t = self.advance()?;

        let expr = match t.token {
            TokenType::Identifier => Expr::Variable(t),
            TokenType::True => Expr::Literal(Literal::Boolean(true)),
            TokenType::False => Expr::Literal(Literal::Boolean(false)),
            TokenType::Nil => Expr::Literal(Literal::Nil),
            TokenType::String => Expr::Literal(t.literal.to_owned()),
            TokenType::Number => Expr::Literal(t.literal.to_owned()),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(&vec![TokenType::RightParen])?;
                Expr::Grouping(Box::new(expr))
            }
            _ => return Err(ParseError::ExpectedExpression(t.line, t.lexeme)),
        };

        Ok(expr)
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Stmt, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.tokens.peek() {
            Some(Ok(token)) => match token.token {
                TokenType::Eof => return None,
                _ => (),
            },
            None => return None,
            _ => (),
        }

        match self.declaration() {
            Ok(s) => Some(Ok(s)),
            Err(e) => {
                self.synchronize();
                Some(Err(e))
            }
        }
    }
}
