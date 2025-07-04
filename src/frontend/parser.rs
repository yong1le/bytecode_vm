use std::{iter::Peekable, vec};

use crate::{
    ast::{expr::Expr, stmt::Stmt},
    core::{
        errors::{InterpretError, SyntaxError},
        token::{Token, TokenType},
    },
    frontend::scanner::Scanner,
};

/// An iterator over the statements in the code.
pub struct Parser<'a> {
    /// An iterator over the tokens in the code.
    tokens: Peekable<Scanner<'a>>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser from the given scanner.
    pub fn new(tokens: Scanner<'a>) -> Self {
        Self {
            tokens: tokens.peekable(),
        }
    }

    /// Advances to the next token to parse. If there are no more tokens to parse,
    /// An `UnexpectedEOF` error is returned, because `advance()` is only called when
    /// the grammar expects another function
    fn advance(&mut self) -> Result<Token, InterpretError> {
        match self.tokens.next() {
            Some(Ok(t)) => Ok(t),
            Some(Err(e)) => Err(e),
            None => Err(InterpretError::Syntax(SyntaxError::UnexpectedEOF)),
        }
    }

    /// Peeks at the next token to parse. If there are no more tokens to parse,
    /// An `UnexpectedEOF` error is returned, because `peek()` is only called when
    /// the grammar expects another function
    fn peek(&mut self) -> Result<&Token, InterpretError> {
        match self.tokens.peek() {
            Some(Ok(t)) => Ok(t),
            Some(Err(e)) => Err(e.to_owned()),
            None => Err(InterpretError::Syntax(SyntaxError::UnexpectedEOF)),
        }
    }

    /// Advances to the next token to parse if the next token is in `tokens`. If
    /// the token is not in `tokens`, an `SyntaxError::ExpectedChar` error is returned.
    fn consume(&mut self, token: TokenType) -> Result<Token, InterpretError> {
        let next_token = match self.tokens.peek() {
            Some(Ok(t)) => t,
            Some(Err(e)) => return Err(e.to_owned()),
            None => return Err(InterpretError::Syntax(SyntaxError::UnexpectedEOF)),
        };

        if token == next_token.token {
            self.advance()
        } else {
            Err(InterpretError::Syntax(SyntaxError::ExpectedChar(
                next_token.line,
                next_token.lexeme.to_owned(),
                format!("{:?}", token),
            )))
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
                Err(InterpretError::Syntax(SyntaxError::UnexpectedEOF)) => return,
                Err(_) => TokenType::Nil, // Anything that doesn't match below should work
            };

            if cur_token == TokenType::Semicolon {
                return;
            }

            let next_token = match self.peek() {
                Ok(t) => &t.token,
                Err(InterpretError::Syntax(SyntaxError::UnexpectedEOF)) => return,
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

    fn declaration(&mut self) -> Result<Stmt, InterpretError> {
        let t = self.peek()?;

        match t.token {
            TokenType::Var => {
                self.advance()?;
                self.declare_var()
            }
            TokenType::Fun => {
                self.advance()?;
                self.declare_func()
            }
            TokenType::Class => {
                self.advance()?;
                self.declare_class()
            }
            _ => self.statement(),
        }
    }

    fn declare_var(&mut self) -> Result<Stmt, InterpretError> {
        let identifier_token = self.consume(TokenType::Identifier)?;

        if let Ok(_equals) = self.consume(TokenType::Equal) {
            let initializer = self.expression()?;
            self.consume(TokenType::Semicolon)?;
            Ok(Stmt::DeclareVar(identifier_token, Some(initializer)))
        } else {
            self.consume(TokenType::Semicolon)?;
            Ok(Stmt::DeclareVar(identifier_token, None))
        }
    }

    fn declare_func(&mut self) -> Result<Stmt, InterpretError> {
        let identifier_token = self.consume(TokenType::Identifier)?;

        let mut params = Vec::new();

        self.consume(TokenType::LeftParen)?;

        loop {
            let t = self.peek()?;

            match t.token {
                TokenType::RightParen | TokenType::Eof => {
                    break;
                }
                _ => {
                    if params.len() >= 255 {
                        return Err(InterpretError::Syntax(SyntaxError::TooManyParams(t.line)));
                    }

                    let param = self.consume(TokenType::Identifier)?;
                    params.push(param);
                    if self.consume(TokenType::Comma).is_err() {
                        break;
                    }
                }
            }
        }
        let closing = self.consume(TokenType::RightParen)?;

        let body = match self.statement()? {
            Stmt::Block(v) => v,
            _ => {
                return Err(InterpretError::Syntax(SyntaxError::ExpectedChar(
                    closing.line,
                    ")".to_string(),
                    "function body".to_string(),
                )))
            }
        };

        Ok(Stmt::DeclareFunc(identifier_token, params, body))
    }

    fn declare_class(&mut self) -> Result<Stmt, InterpretError> {
        let identifier_token = self.consume(TokenType::Identifier)?;
        let mut methods = Vec::new();

        let superclass = if self.consume(TokenType::LessThan).is_ok() {
            Some(self.consume(TokenType::Identifier)?)
        } else {
            None
        };

        self.consume(TokenType::LeftBrace)?;

        loop {
            let t = self.peek()?;

            match t.token {
                TokenType::RightBrace | TokenType::Eof => {
                    break;
                }
                _ => {
                    let method = self.declare_func()?;
                    match method {
                        Stmt::DeclareFunc(id, params, body) => {
                            methods.push((id, params, body));
                        }
                        _ => {
                            // This should never happen
                            panic!("parser.decalre_func() did not return function statement.")
                        }
                    }
                }
            }
        }

        self.consume(TokenType::RightBrace)?;

        Ok(Stmt::DeclareClass(identifier_token, superclass, methods))
    }

    fn statement(&mut self) -> Result<Stmt, InterpretError> {
        let t = self.peek()?;

        match t.token {
            TokenType::Print => {
                let actual = self.advance()?;
                self.print_stmt(actual)
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
            TokenType::Return => {
                let actual = self.advance()?;
                self.return_stmt(actual)
            }
            _ => self.expression_stmt(),
        }
    }

    fn print_stmt(&mut self, token: Token) -> Result<Stmt, InterpretError> {
        let print_expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Print(token, print_expr))
    }

    fn block(&mut self) -> Result<Stmt, InterpretError> {
        let mut statements = vec![];

        loop {
            let token = self.peek()?;
            match token.token {
                TokenType::RightBrace | TokenType::Eof => break,
                _ => statements.push(self.declaration()?),
            }
        }

        self.consume(TokenType::RightBrace)?;
        Ok(Stmt::Block(statements))
    }

    fn if_stmt(&mut self) -> Result<Stmt, InterpretError> {
        // Match the pattern (<condition>)
        let token = self.consume(TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen)?;

        let if_block = self.statement()?;

        if self.consume(TokenType::Else).is_ok() {
            let else_block = self.statement()?;
            Ok(Stmt::If(
                token,
                condition,
                Box::new(if_block),
                Some(Box::new(else_block)),
            ))
        } else {
            Ok(Stmt::If(token, condition, Box::new(if_block), None))
        }
    }

    fn while_stmt(&mut self) -> Result<Stmt, InterpretError> {
        let token = self.consume(TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen)?;

        let while_block = self.statement()?;

        Ok(Stmt::While(token, condition, Box::new(while_block)))
    }

    fn for_stmt(&mut self) -> Result<Stmt, InterpretError> {
        let left_paren = self.consume(TokenType::LeftParen)?;
        let line = left_paren.line;

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
        let right_paren = self.consume(TokenType::RightParen)?;

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Stmt::Block(vec![body, Stmt::Expr(right_paren, inc)]);
        };

        match condition {
            Some(cond) => {
                body = Stmt::While(left_paren, cond, Box::new(body));
            }
            None => {
                body = Stmt::While(
                    left_paren,
                    Expr::Literal(Token {
                        token: TokenType::True,
                        lexeme: "true".to_string(),
                        line,
                    }),
                    Box::new(body),
                );
            }
        };

        if let Some(init) = initializer {
            body = Stmt::Block(vec![init, body]);
        };

        Ok(body)
    }

    fn return_stmt(&mut self, token: Token) -> Result<Stmt, InterpretError> {
        if self.consume(TokenType::Semicolon).is_ok() {
            let line = token.line;
            return Ok(Stmt::Return(
                token,
                Expr::Literal(Token {
                    token: TokenType::Nil,
                    lexeme: "nil".to_string(),
                    line,
                }),
            ));
        }
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Return(token, expr))
    }

    fn expression_stmt(&mut self) -> Result<Stmt, InterpretError> {
        let expr = self.expression()?;
        let token = self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Expr(token, expr))
    }

    fn expression(&mut self) -> Result<Expr, InterpretError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, InterpretError> {
        let expr = self.logic_or()?;

        let t = self.peek()?;

        match t.token {
            TokenType::Equal => {
                let actual = self.advance()?;
                let value = self.assignment()?;

                match expr {
                    Expr::Variable(id) => Ok(Expr::Assign(id, Box::new(value))),
                    Expr::Get(obj, prop) => Ok(Expr::Set(obj, prop, Box::new(value))),
                    _ => Err(InterpretError::Syntax(SyntaxError::InvalidAssignment(
                        actual.line,
                    ))),
                }
            }
            _ => Ok(expr),
        }
    }

    fn logic_or(&mut self) -> Result<Expr, InterpretError> {
        let mut expr = self.logic_and()?;

        loop {
            let t = self.peek()?;

            match t.token {
                TokenType::Or => {
                    let actual = self.advance()?;
                    let right = self.logic_and()?;
                    expr = Expr::Or(actual, Box::new(expr), Box::new(right))
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expr, InterpretError> {
        let mut expr = self.equality()?;

        loop {
            let t = self.peek()?;

            match t.token {
                TokenType::And => {
                    let actual = self.advance()?;
                    let right = self.equality()?;
                    expr = Expr::And(actual, Box::new(expr), Box::new(right))
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, InterpretError> {
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

    fn comparison(&mut self) -> Result<Expr, InterpretError> {
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

    fn term(&mut self) -> Result<Expr, InterpretError> {
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

    fn factor(&mut self) -> Result<Expr, InterpretError> {
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

    fn unary(&mut self) -> Result<Expr, InterpretError> {
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

    fn call(&mut self) -> Result<Expr, InterpretError> {
        let mut expr = self.primary()?;

        loop {
            let mut args = Vec::new();
            if self.consume(TokenType::LeftParen).is_ok() {
                loop {
                    let t = self.peek()?;

                    match t.token {
                        TokenType::RightParen | TokenType::Eof => {
                            break;
                        }
                        _ => {
                            if args.len() >= 255 {
                                return Err(InterpretError::Syntax(SyntaxError::TooManyArgs(
                                    t.line,
                                )));
                            }
                            args.push(self.expression()?);
                            if self.consume(TokenType::Comma).is_err() {
                                break;
                            }
                        }
                    }
                }

                let closing = self.consume(TokenType::RightParen)?;

                expr = Expr::Call(Box::new(expr), args, closing);
            } else if self.consume(TokenType::Dot).is_ok() {
                let prop = self.consume(TokenType::Identifier)?;
                expr = Expr::Get(Box::new(expr), prop);
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, InterpretError> {
        let t = self.advance()?;

        let expr = match &t.token {
            TokenType::Identifier => Expr::Variable(t),
            TokenType::True
            | TokenType::False
            | TokenType::Nil
            | TokenType::String
            | TokenType::Number => Expr::Literal(t),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen)?;
                Expr::Grouping(Box::new(expr))
            }
            TokenType::This => Expr::This(t),
            TokenType::Super => {
                self.consume(TokenType::Dot)?;
                let prop = self.consume(TokenType::Identifier)?;

                Expr::Super(t, prop)
            }
            _ => {
                return Err(InterpretError::Syntax(SyntaxError::ExpectedExpression(
                    t.line, t.lexeme,
                )))
            }
        };

        Ok(expr)
    }
}

impl Iterator for Parser<'_> {
    type Item = Result<Stmt, InterpretError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokens.peek() {
            Some(Ok(token)) => {
                if token.token == TokenType::Eof {
                    return None;
                }
            }
            Some(Err(_)) => (),
            None => return None,
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
