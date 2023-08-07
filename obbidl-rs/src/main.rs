use std::fs;

use ast::{Program, Role};
use generate::generate_rust_bindings;
use parser::parse;

use crate::to_fsm::compile_protocol_def;

mod ast;
mod fsm;
mod generate;
mod lexer;
mod parser;
mod report;
mod to_fsm;
mod token;

fn main() {
    let source = fs::read_to_string("example.txt").unwrap();
    let program = match parse::<Program>(&source) {
        Ok(ast) => ast,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    for protocol in &program.protocols {
        let protocol = compile_protocol_def(protocol);
        let output = generate_rust_bindings(protocol, Role::new("C"), Role::new("S".to_string()));
        fs::write("output/output.rs", &output).unwrap();

        // fs::write(
        //     "output/output.dot",
        //     protocol.state_machine.graph_viz().to_string(),
        // )
        // .unwrap();

        // Command::new("dot")
        //     .arg("-Tsvg")
        //     .arg("-O")
        //     .arg("output/output.dot")
        //     .status()
        //     .unwrap();

        // open::that("output/output.dot.svg").unwrap();
    }
}
