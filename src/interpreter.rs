use std::{error, fmt};

use anyhow::Result;

use crate::{
    ast::{Binary, Expr, Grouping, Literal, Unary, VisitExpr},
    scanner::TokenType, statement::{VisitStmt, Stmt},
};

#[derive(Debug)]
pub struct RuntimeError(pub usize, pub String);

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RuntimeError:\n\n\t[Line {}] {}", self.0, self.1)
    }
}
impl error::Error for RuntimeError {}

// type RuntimeResult<T> = Result<T, RuntimeError>;
pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }
    fn evaluate(&self, expr: &Box<Expr>) -> Result<Literal> {
        return expr.accept(self)
    }

    fn execute(&self, stmt: &Box<Stmt>) -> Result<Option<Literal>> {
        let _ = stmt.accept(self)?;
        return Ok(None);
    }

    pub fn interpret(&self, statements: Vec<Box<Stmt>>) -> Result<Option<Literal>> {
        for stmt in statements {
            self.execute(&stmt)?;
        }
        return Ok(None)
    }
}

impl VisitStmt<Result<Option<Literal>>> for Interpreter {
    fn visit_expr_stmt(&self, expr: &Box<Expr>) -> Result<Option<Literal>> {
        _ = self.evaluate(expr)?;
        return Ok(None)
    }

    fn visit_print_stmt(&self, expr: &crate::statement::PrintStmt) -> Result<Option<Literal>> {
        let value = self.evaluate(&expr.0)?;
        println!("{value}");
        Ok(None)
    }
}

impl VisitExpr<Result<Literal>> for Interpreter {
    fn visit_binary(&self, expr: &Binary) -> Result<Literal> {
        let left = self.evaluate(&expr.0)?;
        let right = self.evaluate(&expr.2)?;

        match expr.1.token_type {
            Some(TokenType::Greater) => {
                let lnum = left.try_num().map_err(|v| RuntimeError(expr.1.line, v))?;
                let rnum = right.try_num().map_err(|v| RuntimeError(expr.1.line, v))?;
                return Ok(Literal::Bool(lnum > rnum));
            }
            Some(TokenType::GreaterEqual) => {
                let lnum = left.try_num().map_err(|v| RuntimeError(expr.1.line, v))?;
                let rnum = right.try_num().map_err(|v| RuntimeError(expr.1.line, v))?;
                return Ok(Literal::Bool(lnum >= rnum));
            }
            Some(TokenType::Less) => {
                let lnum = left.try_num().map_err(|v| RuntimeError(expr.1.line, v))?;
                let rnum = right.try_num().map_err(|v| RuntimeError(expr.1.line, v))?;
                return Ok(Literal::Bool(lnum < rnum));
            }
            Some(TokenType::LessEqual) => {
                let lnum = left.try_num().map_err(|v| RuntimeError(expr.1.line, v))?;
                let rnum = right.try_num().map_err(|v| RuntimeError(expr.1.line, v))?;
                return Ok(Literal::Bool(lnum <= rnum));
            }
            Some(TokenType::Minus) => {
                let lnum = left.try_num().map_err(|v| RuntimeError(expr.1.line, v))?;
                let rnum = right.try_num().map_err(|v| RuntimeError(expr.1.line, v))?;
                return Ok(Literal::Num(lnum - rnum));
            }
            Some(TokenType::Slash) => {
                let lnum = left.try_num().map_err(|v| RuntimeError(expr.1.line, v))?;
                let rnum = right.try_num().map_err(|v| RuntimeError(expr.1.line, v))?;
                return Ok(Literal::Num(lnum / rnum));
            }
            Some(TokenType::Star) => {
                let lnum = left.try_num().map_err(|v| RuntimeError(expr.1.line, v))?;
                let rnum = right.try_num().map_err(|v| RuntimeError(expr.1.line, v))?;
                return Ok(Literal::Num(lnum * rnum));
            }
            Some(TokenType::Plus) => {
                let lnum = left.try_num().map_err(|v| RuntimeError(expr.1.line, v));
                let rnum = right.try_num().map_err(|v| RuntimeError(expr.1.line, v));

                if lnum.is_ok() && rnum.is_ok() {
                    return Ok(Literal::Num(
                        lnum.expect("left number NaN") + rnum.expect("right number NaN"),
                    ));
                }
                let lstr = left
                    .try_string()
                    .map_err(|v| RuntimeError(expr.1.line, v))?;
                let rstr = right
                    .try_string()
                    .map_err(|v| RuntimeError(expr.1.line, v))?;
                return Ok(Literal::Str(format!("{lstr}{rstr}")));
            }
            Some(TokenType::BangEqual) => Ok(Literal::Bool(left.ne(&right))),
            Some(TokenType::EqualEqual) => Ok(Literal::Bool(left.eq(&right))),
            _ => unreachable!("Binary unreachable token_type"),
        }
    }

    fn visit_literal(&self, expr: &Literal) -> Result<Literal> {
        Ok(expr.clone())
    }

    fn visit_unary(&self, expr: &Unary) -> Result<Literal> {
        let right = self.evaluate(&expr.1)?;

        match expr.0.token_type {
            Some(TokenType::Minus) => {
                let rnum = right.try_num().map_err(|v| RuntimeError(expr.0.line, v))?;
                return Ok(Literal::Num(-rnum));
            }
            Some(TokenType::Bang) => {
                return Ok(Literal::Bool(!right.is_truthy()));
            }
            _ => Ok(right),
        }
    }

    fn visit_grouping(&self, expr: &Grouping) -> Result<Literal> {
        self.evaluate(&expr.0)
    }
}
