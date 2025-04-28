use std::{iter::Peekable, vec};

use crate::{
    ast::{expr::Expr, stmt::Stmt},
    core::{
        errors::SyntaxError,
        literal::Literal,
        token::{Token, TokenType},
    },
    scanner::Scanner,
};

// program        →  declaration* EOF;
// declaration    → varDecl | statement;
// varDecl        → "var" IDENTIFIER ( "=" )? ";";
// statement      → exprStmt | printStmt | if | for | while | block ;
// block          → "{" declaration* "}"
// exprStmt       → expression ";" '
// printStmt      → "print" expression ";" ;
// if             → "if (" expression ")" statement ( "else" statement )?
// while          → "while (" expression ")" statement

// expression     → assignment ;
// assignment     → IDENTIFIER "=" assignment | logic_or;
// logic_or       → logic_and ( "or" logic_and )* ;
// logic_and      → equality ( "and" equality )* ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | call ;
// call           → primary ( "(" arguments? ")" )* ;
// arguments      → expression ( "," expression )* ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" | IDENTIFIER;

/// An iterator over the statements in the code.
pub struct Parser<'a> {
    /// An iterator over the tokens in the code.
    tokens: Peekable<Scanner<'a>>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser from the given scanner.
    pub fn new(tokens: Scanner<'a>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    /// Parses the first expression from the list of tokens and advances the
    /// iterator.
    pub fn parse(&mut self) -> Option<Result<Expr, SyntaxError>> {
        match self.tokens.peek() {
            Some(Ok(token)) => {
                if token.token == TokenType::Eof {
                    return None;
                }
            }
            None => return None,
            _ => (),
        }

        let result = self.expression();

        Some(result)
    }

    /// Advances to the next token to parse. If there are no more tokens to parse,
    /// An `UnexpectedEOF` error is returned, because `advance()` is only called when
    /// the grammar expects another function
    fn advance(&mut self) -> Result<Token, SyntaxError> {
        match self.tokens.next() {
            Some(Ok(t)) => Ok(t),
            Some(Err(e)) => Err(SyntaxError::ScanError(e)),
            None => Err(SyntaxError::UnexpectedEOF),
        }
    }

    /// Peeks at the next token to parse. If there are no more tokens to parse,
    /// An `UnexpectedEOF` error is returned, because `peek()` is only called when
    /// the grammar expects another function
    fn peek(&mut self) -> Result<&Token, SyntaxError> {
        match self.tokens.peek() {
            Some(Ok(t)) => Ok(t),
            Some(Err(e)) => Err(SyntaxError::ScanError(e.to_owned())),
            None => Err(SyntaxError::UnexpectedEOF),
        }
    }

    /// Advances to the next token to parse if the next token is in `tokens`. If
    /// the token is not in `tokens`, an `SyntaxError::ExpectedChar` error is returned.
    fn consume(&mut self, token: TokenType) -> Result<Token, SyntaxError> {
        let next_token = match self.tokens.peek() {
            Some(Ok(t)) => t,
            Some(Err(e)) => return Err(SyntaxError::ScanError(e.to_owned())),
            None => return Err(SyntaxError::UnexpectedEOF),
        };

        if token == next_token.token {
            self.advance()
        } else {
            Err(SyntaxError::ExpectedChar(
                next_token.line,
                next_token.lexeme.to_owned(),
                format!("{}", token),
            ))
        }
    }

    /// Synchronizes the parser by discarding tokens until it finds a token that
    /// highly represents the start of a new statement. This is used to recover from
    /// errors.
    fn synchronize(&mut self) {
        // Discard the value, since we know its going to be an error
        self.advance().ok();
        loop {
            let cur_token = match self.advance() {
                Ok(t) => t.token,
                Err(SyntaxError::UnexpectedEOF) => return,
                Err(_) => TokenType::Nil, // Anything that doesn't match below should work
            };

            if cur_token == TokenType::Semicolon {
                return;
            }

            let next_token = match self.peek() {
                Ok(t) => &t.token,
                Err(SyntaxError::UnexpectedEOF) => return,
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

    fn declaration(&mut self) -> Result<Stmt, SyntaxError> {
        let t = self.peek()?;

        match t.token {
            TokenType::Var => {
                self.advance()?;
                self.declare_var()
            }
            _ => self.statement(),
        }
    }

    fn declare_var(&mut self) -> Result<Stmt, SyntaxError> {
        let identifier_token = self.advance()?;

        match identifier_token.token {
            TokenType::Identifier => {
                if let Ok(_equals) = self.consume(TokenType::Equal) {
                    let initializer = self.expression()?;
                    self.consume(TokenType::Semicolon)?;
                    Ok(Stmt::DeclareVar(identifier_token, initializer))
                } else {
                    self.consume(TokenType::Semicolon)?;
                    Ok(Stmt::DeclareVar(
                        identifier_token,
                        Expr::Literal(Literal::Nil),
                    ))
                }
            }
            _ => Err(SyntaxError::ExpectedChar(
                identifier_token.line,
                identifier_token.lexeme.to_string(),
                "IDENTIFIER".to_string(),
            )),
        }
    }

    fn statement(&mut self) -> Result<Stmt, SyntaxError> {
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
            TokenType::If => {
                self.advance()?;
                self.if_stmt()
            }
            TokenType::While => {
                self.advance()?;
                self.while_stmt()
            }
            TokenType::For => {
                self.advance()?;
                self.for_stmt()
            }
            _ => self.expression_stmt(),
        }
    }

    fn print_stmt(&mut self) -> Result<Stmt, SyntaxError> {
        let print_expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Print(print_expr))
    }

    fn block(&mut self) -> Result<Stmt, SyntaxError> {
        let mut statements = vec![];

        loop {
            let token = self.peek()?;
            match token.token {
                TokenType::Eof => {
                    return Err(SyntaxError::ExpectedChar(
                        token.line,
                        "EOF".to_string(),
                        format!("{}", TokenType::Semicolon),
                    ))
                }
                TokenType::RightBrace => break,
                _ => statements.push(self.declaration()?),
            }
        }

        self.consume(TokenType::RightBrace)?;
        Ok(Stmt::Block(statements))
    }

    fn if_stmt(&mut self) -> Result<Stmt, SyntaxError> {
        // Match the pattern (<condition>)
        self.consume(TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen)?;

        let if_block = self.statement()?;

        if self.consume(TokenType::Else).is_ok() {
            let else_block = self.statement()?;
            Ok(Stmt::If(
                condition,
                Box::new(if_block),
                Some(Box::new(else_block)),
            ))
        } else {
            Ok(Stmt::If(condition, Box::new(if_block), None))
        }
    }

    fn while_stmt(&mut self) -> Result<Stmt, SyntaxError> {
        self.consume(TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen)?;

        let while_block = self.statement()?;

        Ok(Stmt::While(condition, Box::new(while_block)))
    }

    fn for_stmt(&mut self) -> Result<Stmt, SyntaxError> {
        self.consume(TokenType::LeftParen)?;

        let initializer = match self.peek()?.token {
            TokenType::Semicolon => {
                self.advance()?;
                None
            }
            TokenType::Var => {
                self.advance()?;
                Some(self.declare_var()?)
            }
            _ => Some(self.expression_stmt()?),
        };

        let condition = match self.peek()?.token {
            TokenType::Semicolon => None,
            _ => Some(self.expression()?),
        };
        self.consume(TokenType::Semicolon)?;

        let increment = match self.peek()?.token {
            TokenType::RightParen => None,
            _ => Some(self.expression()?),
        };
        self.consume(TokenType::RightParen)?;

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Stmt::Block(vec![body, Stmt::Expr(inc)]);
        };

        match condition {
            Some(cond) => {
                body = Stmt::While(cond, Box::new(body));
            }
            None => {
                body = Stmt::While(Expr::Literal(Literal::Boolean(true)), Box::new(body));
            }
        };

        if let Some(init) = initializer {
            body = Stmt::Block(vec![init, body]);
        };

        Ok(body)
    }

    fn expression_stmt(&mut self) -> Result<Stmt, SyntaxError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Expr(expr))
    }

    fn expression(&mut self) -> Result<Expr, SyntaxError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, SyntaxError> {
        let expr = self.logic_or()?;

        let t = self.peek()?;

        match t.token {
            TokenType::Equal => {
                let actual = self.advance()?;
                let value = self.assignment()?;

                match expr {
                    Expr::Variable(id) => Ok(Expr::Assign(id, Box::new(value))),
                    a => Err(SyntaxError::InvalidAssignment(actual.line, a)),
                }
            }
            _ => Ok(expr),
        }
    }

    fn logic_or(&mut self) -> Result<Expr, SyntaxError> {
        let mut expr = self.logic_and()?;

        loop {
            let t = self.peek()?;

            match t.token {
                TokenType::Or => {
                    self.advance()?;
                    let right = self.logic_and()?;
                    expr = Expr::Or(Box::new(expr), Box::new(right))
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expr, SyntaxError> {
        let mut expr = self.equality()?;

        loop {
            let t = self.peek()?;

            match t.token {
                TokenType::And => {
                    self.advance()?;
                    let right = self.equality()?;
                    expr = Expr::And(Box::new(expr), Box::new(right))
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, SyntaxError> {
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

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, SyntaxError> {
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

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, SyntaxError> {
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

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, SyntaxError> {
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

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, SyntaxError> {
        let t = self.peek()?;

        match t.token {
            TokenType::Bang | TokenType::Minus => {
                let op = self.advance()?;
                let expr = self.unary()?;
                Ok(Expr::Unary(op, Box::new(expr)))
            }
            _ => self.call(),
        }
    }

    fn call(&mut self) -> Result<Expr, SyntaxError> {
        let mut expr = self.primary()?;
        let mut args = Vec::new();

        if self.consume(TokenType::LeftParen).is_ok() {
            loop {
                let t = self.peek()?;

                match t.token {
                    TokenType::RightParen => {
                        break;
                    }
                    _ => {
                        args.push(self.expression()?);
                        if self.consume(TokenType::Comma).is_err() {
                            break;
                        }
                    }
                }
            }

            let closing = self.consume(TokenType::RightParen)?;

            expr = Expr::Call(Box::new(expr), args, closing);
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, SyntaxError> {
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
                self.consume(TokenType::RightParen)?;
                Expr::Grouping(Box::new(expr))
            }
            _ => return Err(SyntaxError::ExpectedExpression(t.line, t.lexeme)),
        };

        Ok(expr)
    }
}

impl Iterator for Parser<'_> {
    type Item = Result<Stmt, SyntaxError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokens.peek() {
            Some(Ok(token)) => {
                if token.token == TokenType::Eof {
                    return None;
                }
            }
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
