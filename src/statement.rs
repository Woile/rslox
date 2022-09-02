use crate::ast::Expr;

#[derive(Debug)]
pub enum Stmt {
    Program,
    Statement,
    Expr(Box<Expr>),
    PrintStmt(PrintStmt),
}
#[derive(Debug)]
pub struct ExprStmt;

#[derive(Debug)]
pub struct PrintStmt(pub Box<Expr>);

impl Stmt {

    pub fn accept<T, V: VisitStmt<T>>(&self, visitor: &V) -> T {
        match self {
            Stmt::Program => todo!(),
            Stmt::Statement => todo!(),
            Stmt::Expr(expr) => visitor.visit_expr_stmt(expr),
            Stmt::PrintStmt(print_smt) => visitor.visit_print_stmt(print_smt),
        }
    }
}

pub trait VisitStmt<T> {
    fn visit_expr_stmt(&self, expr: &Box<Expr>) -> T;
    fn visit_print_stmt(&self, expr: &PrintStmt) -> T;
}