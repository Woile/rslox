use crate::{ast::Expr, scanner::Token};

#[derive(Debug)]
pub enum Stmt {
    Expr(Box<Expr>),
    PrintStmt(PrintStmt),
    Var(Var),
}

#[derive(Debug)]
pub struct ExprStmt;

#[derive(Debug)]
pub struct PrintStmt(pub Box<Expr>);

#[derive(Debug)]
pub struct Var {
    pub name: Token,
    pub initializer: Box<Expr>,
}

impl Stmt {
    pub fn accept<T, V: VisitStmt<T>>(&self, visitor: &V) -> T {
        match self {
            Stmt::Expr(expr) => visitor.visit_expr_stmt(expr),
            Stmt::PrintStmt(print_smt) => visitor.visit_print_stmt(print_smt),
            Stmt::Var(var) => visitor.visit_var_stmt(var),
        }
    }
}

pub trait VisitStmt<T> {
    fn visit_expr_stmt(&self, stmt: &Box<Expr>) -> T;
    fn visit_print_stmt(&self, stmt: &PrintStmt) -> T;
    fn visit_var_stmt(&self, stmt: &Var) -> T;
}
