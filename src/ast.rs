use std::fmt::Display;

use crate::scanner::Token;

#[derive(Debug)]
pub enum Expr {
    Binary(Binary),
    Literal(Literal),
    Unary(Unary),
    Grouping(Grouping),
}

impl Expr {
    pub fn accept<T, V: VisitExpr<T>>(&self, visitor: &V) -> T {
        return match self {
            Expr::Binary(binary) => visitor.visit_binary(binary),
            Expr::Literal(literal) => visitor.visit_literal(literal),
            Expr::Unary(unary) => visitor.visit_unary(unary),
            Expr::Grouping(grouping) => visitor.visit_grouping(grouping),
        };
    }
}

#[derive(Debug)]
pub struct Binary(pub Box<Expr>, pub Token, pub Box<Expr>);

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Num(f64),
    Str(String),
    Bool(bool),
    Nil,
}

impl Literal {
    pub fn is_truthy(&self) -> bool {
        match self {
            Literal::Num(_) => true,
            Literal::Str(_) => true,
            Literal::Bool(value) => *value,
            Literal::Nil => false,
        }
    }

    pub fn try_num(&self) -> Result<f64, String> {
        match self {
            Literal::Num(num) => Ok(*num),
            _ => Err(format!("`{}` NaN", self)),
        }
    }

    pub fn try_string(&self) -> Result<String, String> {
        match self {
            Literal::Str(value) => Ok(value.to_string()),
            _ => Err(format!("`{}` Not a String", self)),
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Num(val) => write!(f, "{}", val),
            Literal::Str(val) => write!(f, "{}", val),
            Literal::Bool(val) => write!(f, "{}", val),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug)]
pub struct Unary(pub Token, pub Box<Expr>);

#[derive(Debug)]
pub struct Grouping(pub Box<Expr>);

pub trait VisitExpr<T> {
    fn visit_binary(&self, expr: &Binary) -> T;
    fn visit_literal(&self, expr: &Literal) -> T;
    fn visit_unary(&self, expr: &Unary) -> T;
    fn visit_grouping(&self, expr: &Grouping) -> T;
}

struct AstPrinter;
impl AstPrinter {
    fn parenthesize(&self, name: String, exprs: &[&Expr]) -> String {
        let r: String = exprs
            .iter()
            .map(|e| e.accept(self))
            .collect::<Vec<String>>()
            .join(" ");

        return format!("({name} {r})");
    }

    fn print(&self, exp: Expr) -> () {
        let ast_generated = exp.accept(self);
        println!("{ast_generated}");
    }
}

impl VisitExpr<String> for AstPrinter {
    fn visit_binary(&self, expr: &Binary) -> String {
        self.parenthesize(
            expr.1.lexeme.to_owned(),
            &[expr.0.as_ref(), expr.2.as_ref()],
        )
    }

    fn visit_literal(&self, expr: &Literal) -> String {
        match expr {
            Literal::Num(n) => n.to_string(),
            Literal::Str(s) => s.to_owned(),
            Literal::Bool(b) => format!("{b}"),
            Literal::Nil => "nil".into(),
        }
    }

    fn visit_unary(&self, expr: &Unary) -> String {
        self.parenthesize(expr.0.lexeme.to_owned(), &[expr.1.as_ref()])
    }

    fn visit_grouping(&self, expr: &Grouping) -> String {
        self.parenthesize("group".to_string(), &[expr.0.as_ref()])
    }
}
