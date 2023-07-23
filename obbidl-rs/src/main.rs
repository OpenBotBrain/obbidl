use std::fs;

use parser::{parse, parse_seq};

use crate::compile::{compile_seq, generate_transitions};

mod ast;
mod compile;
mod fsm;
mod lexer;
mod parse;
mod parser;
mod token;

fn main() {
    // let source = fs::read_to_string("example.txt").unwrap();
    // let ast = match parse(&source) {
    //     Ok(ast) => ast,
    //     Err(err) => {
    //         println!("{}", err);
    //         return;
    //     }
    // };
    // println!("{:#?}", ast);

    let seq = parse_seq("{ X from C to S; Y from C to S; }").unwrap();

    let state_machine = compile_seq(&seq);

    println!("{:#?}", state_machine);
}
