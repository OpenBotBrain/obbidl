use std::{env, fs, path::Path};

use ast::{ProtocolFile, Role};
use compile::{compile_protocol, compile_protocol_file};
use generate::generate_rust_bindings;
use parser::parse;

mod ast;
pub mod channel;
mod compile;
mod generate;
mod graph;
mod lexer;
mod parser;
mod report;
mod state_machine;
mod token;

pub fn build(path: impl AsRef<Path>) {
    let path = path.as_ref();
    let out_dir = env::var("OUT_DIR").unwrap();
    let source = fs::read_to_string(path).unwrap();
    let file = match parse::<ProtocolFile>(&source) {
        Ok(ast) => ast,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };
    let file = compile_protocol_file(&file);
    let output = generate_rust_bindings(&file);
    let file_name = path.file_name().unwrap();
    fs::write(Path::new(&out_dir).join(file_name), output).unwrap();
}
