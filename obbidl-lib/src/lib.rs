use std::{
    env, fs,
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
};

use ast::File;
use compile::compile_protocol_file;
use parser::parse;

use crate::{format::binary::Binary, generate::GenerateRust, validate::validate_protocol_file};

pub mod ast;
pub mod channel;
pub mod compile;
pub mod format;
pub mod generate;
mod graph;
mod lexer;
pub mod parser;
mod report;
mod state_machine;
mod token;
pub mod validate;

pub fn build1(source: &str) -> String {
    let file = match parse::<File>(source) {
        Ok(ast) => ast,
        Err(err) => {
            print!("{}", err);
            panic!()
        }
    };
    let file_fsm = compile_protocol_file(&file);
    let file = match validate_protocol_file(&file_fsm, &file.structs) {
        Ok(file) => file,
        Err(errors) => {
            for err in errors {
                println!("{}", err.pretty_print(&source));
            }
            panic!()
        }
    };
    format_rust(&GenerateRust::<Binary>::new(&file).to_string())
}

pub fn build(path: impl AsRef<Path>) {
    let path = path.as_ref();
    println!("cargo:rerun-if-changed={}", path.display());

    let source = fs::read_to_string(path).unwrap();
    let file = match parse::<File>(&source) {
        Ok(ast) => ast,
        Err(err) => {
            print!("{}", err);
            panic!()
        }
    };
    let file_fsm = compile_protocol_file(&file);
    let file = match validate_protocol_file(&file_fsm, &file.structs) {
        Ok(file) => file,
        Err(errors) => {
            for err in errors {
                println!("{}", err.pretty_print(&source));
            }
            panic!()
        }
    };
    let output = GenerateRust::<Binary>::new(&file).to_string();
    let file_name = path.file_name().unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    fs::write(
        Path::new(&out_dir).join(file_name).with_extension("rs"),
        output,
    )
    .unwrap();
}

pub fn format_rust(string: &str) -> String {
    let child = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.unwrap().write_all(string.as_bytes()).unwrap();
    let mut buffer = vec![];
    child.stdout.unwrap().read_to_end(&mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
