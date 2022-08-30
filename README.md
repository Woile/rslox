# rslox

[Rust](https://www.rust-lang.org/) implementation of the Jlox language described in the book [Crafting Interpreters by Robert Nystrom](https://craftinginterpreters.com/contents.html)

## Features

- [x] Scanner
- [x] Parser
- [x] AST
- [x] Interpreter (Part I)
- [ ] Interpreter (Part II - Statements and State)

## Notes

This is a work in progress, the focus has not been code quality but learning.
I'm quite happy with the results so far.

## Installation

```sh
cargo install
```

## Usage

### Interpreter

```sh
cargo run
```

### Files

```sh
cargo run {file_name}.lox
```

Example

```sh
cargo run tests/scanning/keywords.lox
```
