#![warn(clippy::pedantic)]
mod ast;
mod interpreter;
mod parser;
mod scanner;

use anyhow::Result;

use clap::Parser;

use std::io::Write;
use std::path::PathBuf;
use std::{error, fmt, fs, io};

use interpreter::Interpreter;

// struct Jlox;

#[derive(Debug)]
pub struct JLoxError(usize, String);

impl fmt::Display for JLoxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error:\nline {}: {}", self.0, self.1)
    }
}
impl error::Error for JLoxError {}

type JloxResult = Result<(), Box<dyn error::Error>>;

#[derive(Parser, Debug)]
#[clap(name = "rslox", version, about)]
struct Args {
    /// Output the tokens tree generated
    #[clap(short('t'), long, action)]
    print_tokens: bool,

    /// Output the ast tree generated
    #[clap(short('a'), long, action)]
    print_ast: bool,

    /// Lox files
    #[clap(required = false, parse(from_os_str))]
    path: Option<PathBuf>,
}

fn main() -> JloxResult {
    let args = Args::parse();

    if let Some(path) = &args.path {
        run_file(path, &args)?
    } else {
        run_prompt(&args)?
    }

    Ok(())
}

fn run_prompt(args: &Args) -> JloxResult {
    loop {
        print!("> ");
        io::stdout().flush().expect("Could not flush");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if let Some(err) = run(input, args).err() {
            eprintln!("{:?}", err);
        };
    }
}

fn run_file(filepath: &PathBuf, args: &Args) -> JloxResult {
    let source = fs::read_to_string(filepath)?;
    if let Some(err) = run(source, args).err() {
        eprintln!("{:?}", err);

        std::process::exit(65)
    };
    Ok(())
}

fn run(source: String, args: &Args) -> Result<()> {
    let mut scanner = scanner::Scanner::new(source);
    let _ = scanner.scan_tokens();
    if args.print_tokens {
        println!("{:#?}", scanner.tokens);
    }
    let mut parser = parser::Parser::new(scanner.tokens);
    let expr = parser.parse()?;

    if args.print_ast {
        println!("{:#?}", expr);
    }
    let interp = Interpreter::new();
    let lit = interp.evaluate(&expr)?;
    println!("{}", lit);
    Ok(())
}
