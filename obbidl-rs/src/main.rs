use std::fs;

use parser::parse;

mod ast;
mod lexer;
mod parser;
mod token;

fn main() {
    let source = fs::read_to_string("example.txt").unwrap();
    let ast = match parse(&source) {
        Ok(ast) => ast,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };
    println!("{:#?}", ast);
}
