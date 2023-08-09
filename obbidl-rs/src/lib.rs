use std::{env, fs, path::Path};

use ast::ProtocolFile;
use compile::compile_protocol_file;
use generate::generate_rust_bindings;
use parser::parse;

use crate::{generate::validate_protocol_file, test::GenerateRust};

mod ast;
pub mod channel;
mod compile;
mod generate;
mod graph;
mod lexer;
mod parser;
mod report;
mod state_machine;
mod test;
mod token;

pub fn build(path: impl AsRef<Path>) {
    let path = path.as_ref();
    println!("cargo:rerun-if-changed={}", path.display());

    let out_dir = env::var("OUT_DIR").unwrap();
    let source = fs::read_to_string(path).unwrap();
    let file = match parse::<ProtocolFile>(&source) {
        Ok(ast) => ast,
        Err(err) => {
            print!("{}", err);
            panic!()
        }
    };
    let file = compile_protocol_file(&file);
    let file = validate_protocol_file(&file);
    let output = GenerateRust(&file).to_string();
    let file_name = path.file_name().unwrap();

    fs::write(
        Path::new(&out_dir).join(file_name).with_extension("rs"),
        output,
    )
    .unwrap();
}
