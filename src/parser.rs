use crate::{
    ast::{Binary, Expr, Grouping, Literal, Unary},
    scanner::{Token, TokenType},
};

use std::{
    error,
    fmt::{self},
};

#[derive(Debug)]
pub struct ParserError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ParserError:\n\n\t[Line {}] {}", self.line, self.message)
    }
}
impl error::Error for ParserError {}

type ParserResult<T> = Result<T, ParserError>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// We flip the design proposed in crafting interpreters
    /// This includes the functions `advance`, `check`, `isAtEnd`
    /// `peek` and `previous`
    pub fn parse(&mut self) -> ParserResult<Box<Expr>> {
        self.expression()
    }

    fn expression(&mut self) -> ParserResult<Box<Expr>> {
        return self.equality();
    }

    fn equality(&mut self) -> ParserResult<Box<Expr>> {
        let mut expr = self.comparison()?;
        while self.fits(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Box::new(Expr::Binary(Binary(expr, operator, right)));
        }
        return Ok(expr);
    }

    fn comparison(&mut self) -> ParserResult<Box<Expr>> {
        let mut expr = self.term()?;
        while self.fits(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Box::new(Expr::Binary(Binary(expr, operator, right)));
        }
        return Ok(expr);
    }
    fn term(&mut self) -> ParserResult<Box<Expr>> {
        let mut expr = self.factor()?;
        while self.fits(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Box::new(Expr::Binary(Binary(expr, operator, right)));
        }
        return Ok(expr);
    }

    fn factor(&mut self) -> ParserResult<Box<Expr>> {
        let mut expr = self.unary()?;
        while self.fits(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Box::new(Expr::Binary(Binary(expr, operator, right)));
        }
        return Ok(expr);
    }

    fn unary(&mut self) -> ParserResult<Box<Expr>> {
        if self.fits(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Box::new(Expr::Unary(Unary(operator, right))));
        }
        return self.primary();
    }

    fn primary(&mut self) -> ParserResult<Box<Expr>> {
        if self.fits(vec![TokenType::False]) {
            return Ok(Box::new(Expr::Literal(Literal::Bool(false))));
        }
        if self.fits(vec![TokenType::True]) {
            return Ok(Box::new(Expr::Literal(Literal::Bool(true))));
        }
        if self.fits(vec![TokenType::Nil]) {
            return Ok(Box::new(Expr::Literal(Literal::Nil)));
        }
        if self.fits(vec![TokenType::String]) {
            let val = self.previous().literal.clone();
            return Ok(Box::new(Expr::Literal(Literal::Str(val))));
        }
        if self.check_is_num() {
            if let Some(TokenType::Number(num)) = self.previous().token_type {
                return Ok(Box::new(Expr::Literal(Literal::Num(num))));
            }
        }
        if self.fits(vec![TokenType::LeftParen]) {
            let expr = self.expression()?;
            let _ = self.consume(TokenType::RightParen, "Expect ')' after expression.".into());
            return Ok(Box::new(Expr::Grouping(Grouping(expr))));
        }

        Err(ParserError {
            line: 1,
            message: "Something went wrong".into(),
        })
    }

    fn fits(&mut self, token_types: Vec<TokenType>) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        return false;
    }
    fn check_is_num(&mut self) -> bool {
        if self.is_at_end() {
            return false;
        }
        match self.peek().token_type {
            Some(TokenType::Number(_)) => {
                self.advance();
                true
            }
            _ => false,
        }
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        return self.peek().token_type == Some(token_type);
    }
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        return &self.previous();
    }

    fn is_at_end(&self) -> bool {
        return self.peek().token_type == Some(TokenType::EOF);
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        if self.current == 0 {
            return &self.tokens[0];
        }
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token_type: TokenType, message: String) -> ParserResult<&Token> {
        if self.check(token_type) {
            return Ok(self.advance());
        }
        let line_num = self.peek().line;
        Err(ParserError {
            line: line_num,
            message,
        })
    }

    fn synchronize(&mut self) -> () {
        let _ = self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == Some(TokenType::Semicolon) {
                return;
            }
            match self.peek().token_type {
                Some(
                    TokenType::Class
                    | TokenType::Fun
                    | TokenType::Var
                    | TokenType::For
                    | TokenType::If
                    | TokenType::While
                    | TokenType::Print
                    | TokenType::Return,
                ) => return,
                _ => {
                    self.advance();
                }
            }
        }
    }
}
