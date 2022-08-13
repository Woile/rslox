use clap::Parser;
use multipeek::{multipeek, MultiPeek};
use nom::number::complete::double;
use nom::IResult;

use std::io::Write;
use std::iter::Enumerate;
use std::path::PathBuf;
use std::str::Chars;
use std::{error, fmt, fs, io};

// struct Jlox;

#[derive(Debug)]
struct JLoxError(usize, String);

impl fmt::Display for JLoxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Line {}] Error: {}", self.0, self.1)
    }
}
impl error::Error for JLoxError {}

type JloxResult = Result<(), Box<dyn error::Error>>;

#[derive(Parser, Debug)]
#[clap(name = "rslox", version, about)]
struct Args {
    /// Output the ast tree generated
    #[clap(short, long, action)]
    print_ast: bool,

    /// Lox files
    #[clap(required = false, parse(from_os_str))]
    path: Option<PathBuf>,
}

fn main() -> JloxResult {
    let args = Args::parse();

    if let Some(path) = args.path {
        if args.print_ast {
            let _ = ast::run_ast_example();
        }
        run_file(path)?
    } else {
        run_prompt()?
    }

    Ok(())
}

fn run_prompt() -> JloxResult {
    loop {
        print!("> ");
        io::stdout().flush().expect("Could not flush");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if let Some(err) = run(input).err() {
            eprintln!("{}", err);
        };
    }
}

fn run_file(filepath: PathBuf) -> JloxResult {
    let source = fs::read_to_string(filepath)?;
    if let Some(err) = run(source).err() {
        eprintln!("{}", err);
        std::process::exit(65)
    };
    Ok(())
}

fn run(source: String) -> Result<(), JLoxError> {
    let mut scanner = Scanner {
        source,
        tokens: Vec::new(),
        line: 1,
        start: 0,
        current: 0,
    };
    let _ = scanner.scan_tokens();
    println!("{:#?}", scanner.tokens);
    let mut parser = parser::Parser::new(scanner.tokens);
    let expr = parser.parse();
    println!("-------------");
    println!("{:#?}", expr);

    Ok(())
}

type NumberMetadata = f64;

#[derive(Debug)]
struct TokenInfo {
    token_type: TokenType,
    literal: String,
    new_lines: usize,
}

impl TokenInfo {
    fn new(token_type: TokenType, literal: String, new_lines: usize) -> Self {
        Self {
            token_type,
            literal,
            new_lines,
        }
    }
    fn get(keyword: &str) -> TokenType {
        match keyword {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier,
        }
    }
}

// Scanner part
#[derive(Debug, PartialEq, Clone)]
#[allow(unused)]
pub enum TokenType {
    // Single character token
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or more character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    // Literals
    Identifier,
    String,
    Number(NumberMetadata),
    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    EOF,
}

#[derive(Default, Debug, Clone)]
pub struct Token {
    token_type: Option<TokenType>,
    lexeme: String,
    literal: String,
    line: usize,
}

impl ToString for Token {
    fn to_string(&self) -> String {
        format!("{:?}  {}  {:?}", self.token_type, self.lexeme, self.literal)
    }
}
struct Scanner {
    source: String,
    tokens: Vec<Token>,
    line: usize,
    start: usize,
    current: usize,
}

impl Scanner {
    fn scan_tokens(&mut self) -> Result<(), JLoxError> {
        // let mut tokens = Vec::new();
        let mut iter = multipeek(self.source.chars().enumerate());

        while let Some((current, symbol)) = iter.next() {
            self.current = current;
            if symbol == '\n' {
                self.line += 1;
                // No need for now to do anything else if we know is a new line
                continue;
            }
            let scan_result = self.scan_token(symbol, &mut iter);
            match scan_result {
                Ok(Some(token_info)) => {
                    let token = Token {
                        token_type: Some(token_info.token_type),
                        lexeme: String::new(),
                        literal: token_info.literal,
                        line: self.line,
                    };
                    self.line += token_info.new_lines;
                    self.tokens.push(token)
                }
                Ok(None) => {
                    continue;
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
        let t = Token {
            token_type: Some(TokenType::EOF),
            lexeme: String::from(""),
            literal: String::new(),
            line: self.line,
        };

        self.tokens.push(t);
        return Ok(());
    }

    fn scan_token(
        &self,
        symbol: char,
        iter: &mut MultiPeek<Enumerate<Chars>>,
    ) -> Result<Option<TokenInfo>, JLoxError> {
        // Returning Ok(None) means keep advancing
        match symbol {
            '"' => {
                // String
                let mut value = String::new();
                let mut new_lines_count: usize = 0;
                loop {
                    match iter.peek() {
                        Some((_, '"')) => {
                            // When it's new line break
                            // iter.next();
                            break;
                        }
                        Some((_, '\n')) => {
                            // TODO: move line +1
                            new_lines_count += 1;
                            value.push('\n');
                            iter.next();
                        }
                        Some((_, char)) => {
                            value.push(*char);
                            iter.next();
                        }
                        None => {
                            return Err(JLoxError(self.line, "Unterminated string".into()));
                        }
                    }
                }
                iter.next();
                Ok(Some(TokenInfo::new(
                    TokenType::String,
                    value,
                    new_lines_count,
                )))
            }
            '0'..='9' => {
                // numbers
                let mut literal = String::from(symbol);
                loop {
                    match iter.peek() {
                        Some((_, '1'..='9')) => {
                            let next_char = iter.next();
                            if let Some((_, ch)) = next_char {
                                literal.push(ch);
                            }
                        }
                        Some((_, '.')) => {
                            let peeked = iter.peek();
                            if let Some((_, '1'..='9')) = peeked {
                                let next_char = iter.next();
                                if let Some((_, ch)) = next_char {
                                    literal.push(ch);
                                }
                            } else {
                                return Err(JLoxError(
                                    self.line,
                                    "Error at end: Expect property name after '.'".into(),
                                ));
                            }
                        }
                        _ => {
                            // iter.next();
                            break;
                        }
                    }
                }
                let parsed: IResult<_, _> = double(literal.as_str());
                match parsed {
                    Ok((_, number)) => {
                        return Ok(Some(TokenInfo::new(TokenType::Number(number), literal, 0)));
                    }
                    Err(error) => {
                        return Err(JLoxError(
                            self.line,
                            format!("Problem parsing: {:?}", error),
                        ))
                    }
                }
            }
            'a'..='z' | '_' | 'A'..='Z' => {
                let mut keyword = String::from(symbol);
                loop {
                    match iter.peek() {
                        Some((_, '1'..='9' | '_' | 'a'..='z' | 'A'..='Z')) => {
                            let next_char = iter.next();
                            if let Some((_, ch)) = next_char {
                                keyword.push(ch);
                            }
                        }
                        _ => {
                            iter.next();
                            break;
                        }
                    }
                }
                let token_type = TokenInfo::get(&keyword);
                Ok(Some(TokenInfo::new(token_type, keyword, 0)))
            }
            '(' => Ok(Some(TokenInfo::new(TokenType::LeftParen, symbol.into(), 0))),
            ')' => Ok(Some(TokenInfo::new(
                TokenType::RightParen,
                symbol.into(),
                0,
            ))),
            '{' => Ok(Some(TokenInfo::new(TokenType::LeftBrace, symbol.into(), 0))),

            '}' => Ok(Some(TokenInfo::new(
                TokenType::RightBrace,
                symbol.into(),
                0,
            ))),
            ',' => Ok(Some(TokenInfo::new(TokenType::Comma, symbol.into(), 0))),
            '.' => Ok(Some(TokenInfo::new(TokenType::Dot, symbol.into(), 0))),
            '-' => Ok(Some(TokenInfo::new(TokenType::Minus, symbol.into(), 0))),
            '+' => Ok(Some(TokenInfo::new(TokenType::Plus, symbol.into(), 0))),
            ';' => Ok(Some(TokenInfo::new(TokenType::Semicolon, symbol.into(), 0))),
            '*' => Ok(Some(TokenInfo::new(TokenType::Star, symbol.into(), 0))),
            '!' => match iter.peek() {
                Some((_, '=')) => {
                    iter.next();
                    Ok(Some(TokenInfo::new(TokenType::BangEqual, symbol.into(), 0)))
                }
                _ => Ok(Some(TokenInfo::new(TokenType::Bang, symbol.into(), 0))),
            },
            '=' => match iter.peek() {
                Some((_, '=')) => {
                    iter.next();
                    Ok(Some(TokenInfo::new(
                        TokenType::EqualEqual,
                        symbol.into(),
                        0,
                    )))
                }
                _ => Ok(Some(TokenInfo::new(TokenType::Equal, symbol.into(), 0))),
            },
            '<' => match iter.peek() {
                Some((_, '=')) => {
                    iter.next();
                    Ok(Some(TokenInfo::new(TokenType::LessEqual, symbol.into(), 0)))
                }
                _ => Ok(Some(TokenInfo::new(TokenType::Less, symbol.into(), 0))),
            },
            '>' => match iter.peek() {
                Some((_, '=')) => {
                    iter.next();
                    Ok(Some(TokenInfo::new(
                        TokenType::GreaterEqual,
                        symbol.into(),
                        0,
                    )))
                }
                _ => Ok(Some(TokenInfo::new(TokenType::Greater, symbol.into(), 0))),
            },
            '/' => match iter.peek() {
                Some((_, '/')) => {
                    loop {
                        match iter.peek() {
                            Some((_, '\n')) => {
                                // When it's new line break
                                break;
                            }
                            None => {
                                break;
                            }
                            _ => {
                                // Anything else is a comment
                                iter.next();
                            }
                        }
                    }
                    Ok(None)
                }
                _ => Ok(Some(TokenInfo::new(TokenType::Slash, symbol.into(), 0))),
            },
            ' ' | '\r' | '\t' => Ok(None),
            _ => Err(JLoxError(
                self.line,
                format!("Unexpected character {symbol}").into(),
            )),
        }
    }
}

mod ast {
    use crate::{JloxResult, Token, TokenType};

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

    #[derive(Debug)]
    pub enum Literal {
        Num(f64),
        Str(String),
        Bool(bool),
        Nil(String),
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
                Literal::Nil(n) => n.to_owned(),
            }
        }

        fn visit_unary(&self, expr: &Unary) -> String {
            self.parenthesize(expr.0.lexeme.to_owned(), &[expr.1.as_ref()])
        }

        fn visit_grouping(&self, expr: &Grouping) -> String {
            self.parenthesize("group".to_string(), &[expr.0.as_ref()])
        }
    }

    pub fn run_ast_example() -> JloxResult {
        // Expr::Binary(Binary {

        let exp = Expr::Binary(Binary(
            Box::new(Expr::Unary(Unary(
                Token {
                    token_type: Some(TokenType::Minus),
                    lexeme: String::from("-"),
                    literal: String::from("-"),
                    line: 1,
                },
                Box::new(Expr::Literal(Literal::Num(123.0))),
            ))),
            Token {
                token_type: Some(TokenType::Star),
                lexeme: String::from("*"),
                literal: String::from("*"),
                line: 1,
            },
            Box::new(Expr::Grouping(Grouping(Box::new(Expr::Literal(
                Literal::Num(45.67),
            ))))),
        ));

        let ast = AstPrinter {};
        ast.print(exp);
        // });
        Ok(())
    }
}

mod parser {
    use crate::{
        ast::{Binary, Expr, Unary, Literal, Grouping},
        Token, TokenType,
    };
    use std::{
        error,
        fmt::{self},
    };

    #[derive(Debug)]
    pub struct ParserError(usize, String);

    impl fmt::Display for ParserError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "[Line {}] Parser Error: {}", self.0, self.1)
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
            if self.fits(vec![TokenType::False]){
                return Ok(Box::new(Expr::Literal(Literal::Bool(false))));
            }
            if self.fits(vec![TokenType::True]){
                return Ok(Box::new(Expr::Literal(Literal::Bool(true))));
            }
            if self.fits(vec![TokenType::Nil]){
                return Ok(Box::new(Expr::Literal(Literal::Nil("nil".into()))));
            }
            if self.fits(vec![TokenType::String]){
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
                return Ok(Box::new(Expr::Grouping(Grouping(expr))))
            }


            Err(ParserError(1, "Something went wrong".into()))
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
        fn check_is_num(&self) -> bool {
            if self.is_at_end() {
                return false;
            }
            match self.peek().token_type {
                Some(TokenType::Number(_)) => true,
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
            Err(ParserError(0, message))
        }
    }
}
