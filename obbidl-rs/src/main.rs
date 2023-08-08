use std::fs;

use ast::ProtocolFile;
use compile::compile_protocol_file;
use generate::generate_rust_bindings;
use parser::parse;

mod ast;
mod channel;
mod compile;
mod generate;
mod graph;
mod lexer;
mod parser;
mod report;
mod state_machine;
mod token;

fn main() {
    let source = fs::read_to_string("example.txt").unwrap();
    let file = match parse::<ProtocolFile>(&source) {
        Ok(ast) => ast,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };
    let file = compile_protocol_file(&file);
    let output = generate_rust_bindings(&file);
    fs::write("output.rs", output).unwrap();
}
