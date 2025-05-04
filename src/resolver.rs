use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::{
        expr::{Expr, ExprVisitor},
        stmt::{Stmt, StmtVisitor},
    },
    core::{errors::SemanticError, literal::Literal, token::Token},
    runtime::interpreter::Interpreter,
};

/// A struct that resolves identifiers to its correct scope. Does this to all identifers except
/// those defined in the global scope.
pub struct Resolver<'a> {
    /// The interpreter that will interpret this AST after all identifiers have been resolved
    interpreter: &'a mut Interpreter,
    /// Stack that represents the scope we are in
    scopes: Vec<HashMap<String, bool>>,
    /// The level of nested function declarations we are in
    func_level: u32,
    /// The level of nested class declarations we are in
    class_level: u32,
    /// The level of nested class constructor declarations we are in
    init_level: u32,
    /// The level of nested subclass declarations we are in
    subclass_level: u32,
}

impl<'a> Resolver<'a> {
    /// Creates a new copy of `Resolver`
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            func_level: 0,
            class_level: 0,
            init_level: 0,
            subclass_level: 0,
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), SemanticError> {
        expr.accept(self)
    }

    pub fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<(), SemanticError> {
        stmt.accept(self)
    }

    /// Resolve a visit of a local variable by binding it to the first scope that
    /// defines `lexeme`
    fn resolve_local(&mut self, token_id: usize, lexeme: &str) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.get(lexeme).is_some() {
                self.interpreter
                    .resolve(token_id, (self.scopes.len() - 1 - i) as u32);
                return;
            }
        }
    }

    /// Resolve a functions parameters and body
    fn resolve_function(&mut self, params: &[Token], body: &[Stmt]) -> Result<(), SemanticError> {
        self.begin_scope();

        for param in params {
            self.declare(param)?;
            self.define(param);
        }

        self.func_level += 1;
        for stmt in body {
            self.resolve_stmt(stmt)?;
        }
        self.func_level -= 1;
        self.end_scope();

        Ok(())
    }

    /// Begin a new scope
    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// End the current scope
    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    /// Declares a new uninitialized local variable. Local variables cannot have
    /// multiply declarations.
    fn declare(&mut self, id: &Token) -> Result<(), SemanticError> {
        let scope = self.scopes.last_mut();
        if let Some(s) = scope {
            if s.contains_key(&id.lexeme) {
                return Err(SemanticError::AlreadyDeclared(
                    id.line,
                    id.lexeme.to_owned(),
                ));
            }
            s.insert(id.lexeme.to_owned(), false);
        }

        Ok(())
    }

    /// Sets a local variable to be initialized.
    fn define(&mut self, id: &Token) {
        let scope = self.scopes.last_mut();
        if let Some(s) = scope {
            s.insert(id.lexeme.to_owned(), true);
        }
    }
}

impl ExprVisitor<Result<(), SemanticError>> for Resolver<'_> {
    fn visit_literal(&mut self, _literal: &Literal) -> Result<(), SemanticError> {
        Ok(())
    }

    fn visit_unary(&mut self, _operator: &Token, expr: &Expr) -> Result<(), SemanticError> {
        self.resolve_expr(expr)?;
        Ok(())
    }

    fn visit_binary(
        &mut self,
        _operator: &Token,
        left: &Expr,
        right: &Expr,
    ) -> Result<(), SemanticError> {
        self.resolve_expr(left)?;
        self.resolve_expr(right)?;
        Ok(())
    }

    fn visit_grouping(&mut self, expr: &Expr) -> Result<(), SemanticError> {
        self.resolve_expr(expr)?;
        Ok(())
    }

    fn visit_variable(&mut self, id: &Token) -> Result<(), SemanticError> {
        if let Some(s) = self.scopes.last() {
            if s.get(&id.lexeme) == Some(&false) {
                return Err(SemanticError::UndeclaredLocalInInitializer(id.line));
            }
            self.resolve_local(id.id, &id.lexeme);
            Ok(())
        } else {
            Ok(())
        }
    }

    fn visit_assignment(&mut self, id: &Token, assignment: &Expr) -> Result<(), SemanticError> {
        self.resolve_expr(assignment)?;
        self.resolve_local(id.id, &id.lexeme);

        Ok(())
    }

    fn visit_and(&mut self, left: &Expr, right: &Expr) -> Result<(), SemanticError> {
        self.resolve_expr(left)?;
        self.resolve_expr(right)?;
        Ok(())
    }

    fn visit_or(&mut self, left: &Expr, right: &Expr) -> Result<(), SemanticError> {
        self.resolve_expr(left)?;
        self.resolve_expr(right)?;
        Ok(())
    }

    fn visit_call(
        &mut self,
        callee: &Expr,
        arguments: &[Expr],
        _closing: &Token,
    ) -> Result<(), SemanticError> {
        self.resolve_expr(callee)?;

        for arg in arguments {
            self.resolve_expr(arg)?;
        }
        Ok(())
    }

    fn visit_get(&mut self, obj: &Expr, _: &Token) -> Result<(), SemanticError> {
        self.resolve_expr(obj)?;
        Ok(())
    }

    fn visit_set(&mut self, obj: &Expr, _: &Token, value: &Expr) -> Result<(), SemanticError> {
        self.resolve_expr(obj)?;
        self.resolve_expr(value)?;
        Ok(())
    }

    fn visit_this(&mut self, token: &Token) -> Result<(), SemanticError> {
        if self.class_level == 0 {
            return Err(SemanticError::TopThis(token.line));
        }
        self.resolve_local(token.id, &token.lexeme);
        Ok(())
    }

    fn visit_super(&mut self, super_token: &Token, _: &Token) -> Result<(), SemanticError> {
        if self.class_level == 0 {
            return Err(SemanticError::TopSuper(super_token.line));
        }
        if self.subclass_level == 0 {
            return Err(SemanticError::TopClassSuper(super_token.line));
        }

        self.resolve_local(super_token.id, &super_token.lexeme);
        Ok(())
    }
}

impl StmtVisitor<Result<(), SemanticError>> for Resolver<'_> {
    fn visit_print(&mut self, stmt: &Expr) -> Result<(), SemanticError> {
        self.resolve_expr(stmt)?;
        Ok(())
    }

    fn visit_expr(&mut self, expr: &Expr) -> Result<(), SemanticError> {
        self.resolve_expr(expr)?;
        Ok(())
    }

    fn visit_declare_var(&mut self, id: &Token, expr: &Option<Expr>) -> Result<(), SemanticError> {
        self.declare(id)?;
        if let Some(ex) = expr {
            self.resolve_expr(ex)?;
        }
        self.define(id);
        Ok(())
    }

    fn visit_block(&mut self, statements: &[Stmt]) -> Result<(), SemanticError> {
        self.begin_scope();
        for stmt in statements {
            self.resolve_stmt(stmt)?
        }
        self.end_scope();

        Ok(())
    }

    fn visit_if(
        &mut self,
        condition: &Expr,
        if_block: &Stmt,
        else_block: &Option<Box<Stmt>>,
    ) -> Result<(), SemanticError> {
        self.resolve_expr(condition)?;
        self.resolve_stmt(if_block)?;

        if let Some(else_b) = else_block {
            self.resolve_stmt(else_b)?;
        }

        Ok(())
    }

    fn visit_while(&mut self, condition: &Expr, while_block: &Stmt) -> Result<(), SemanticError> {
        self.resolve_expr(condition)?;
        self.resolve_stmt(while_block)?;

        Ok(())
    }

    fn visit_declare_func(
        &mut self,
        id: &Token,
        params: &Rc<Vec<Token>>,
        body: &Rc<Vec<Stmt>>,
    ) -> Result<(), SemanticError> {
        self.declare(id)?;
        self.define(id);

        self.resolve_function(params, body)?;
        Ok(())
    }

    fn visit_return(&mut self, expr: &Expr, line: &u32) -> Result<(), SemanticError> {
        if self.func_level == 0 {
            return Err(SemanticError::TopReturn(line.to_owned()));
        }

        if self.init_level != 0 {
            match expr {
                Expr::Literal(Literal::Nil) => (),
                _ => return Err(SemanticError::ReturnValueInInit(line.to_owned())),
            }
        }

        self.resolve_expr(expr)?;
        Ok(())
    }

    fn visit_declare_class(
        &mut self,
        id: &Token,
        parent: &Option<Token>,
        methods: &[(Token, Rc<Vec<Token>>, Rc<Vec<Stmt>>)],
    ) -> Result<(), SemanticError> {
        self.declare(id)?;
        self.define(id);

        let mut in_subclass = false;
        if let Some(parent) = parent {
            if parent.lexeme == id.lexeme {
                return Err(SemanticError::SelfInheritance(
                    id.line,
                    id.lexeme.to_string(),
                ));
            } else {
                self.visit_variable(parent)?;

                self.begin_scope();
                if let Some(scope) = self.scopes.last_mut() {
                    scope.insert("super".to_string(), true);
                }

                in_subclass = true;
                self.subclass_level += 1;
            }
        } else {
            self.begin_scope();
        }

        if let Some(scope) = self.scopes.last_mut() {
            scope.insert("this".to_string(), true);
        }

        self.class_level += 1;
        for (name, params, body) in methods {
            if name.lexeme == "init" {
                self.init_level += 1;
            }
            self.resolve_function(params, body)?;
            if name.lexeme == "init" {
                self.init_level -= 1;
            }
        }
        self.class_level -= 1;
        if in_subclass {
            self.subclass_level -= 1;
        }

        self.end_scope();
        Ok(())
    }
}
