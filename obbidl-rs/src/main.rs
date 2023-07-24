use std::{fs, process::Command};

use parser::parse;

use crate::{compile::compile_seq, fsm::GraphViz};

mod ast;
mod compile;
mod fsm;
mod lexer;
mod parser;
mod token;

fn main() {
    let source = fs::read_to_string("example.txt").unwrap();
    let seq = match parse(&source) {
        Ok(ast) => ast,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let state_machine = compile_seq(&seq);

    fs::write("output.dot", GraphViz(state_machine).to_string()).unwrap();

    Command::new("dot")
        .arg("-Tsvg")
        .arg("-O")
        .arg("output.dot")
        .status()
        .unwrap();
}
