use std::{
    env::{self, temp_dir},
    fs,
    io::{Read, Write},
    process::{Command, ExitCode, Stdio},
};

use ast::ProtocolFile;
use compile::compile_protocol_file;
use generate::GenerateRust;
use parser::parse;
use validate::validate_protocol_file;

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
mod validate;

fn main() -> ExitCode {
    let mut graph = false;
    let mut path = None;
    let mut svg = false;
    let mut output_path = None;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--graph" => graph = true,
            "--svg" => svg = true,
            arg => {
                if let Some(arg) = arg.strip_prefix("--output=") {
                    output_path = Some(arg.to_string());
                } else {
                    path = Some(arg.to_string());
                }
            }
        }
    }

    let Some(path) = path else {
        println!("missing input file");
        return ExitCode::FAILURE;
    };

    let source = fs::read_to_string(path).unwrap();
    let file = match parse::<ProtocolFile>(&source) {
        Ok(ast) => ast,
        Err(err) => {
            println!("{}", err);
            return ExitCode::FAILURE;
        }
    };
    let file = compile_protocol_file(&file);

    let output = if graph {
        let graph = file.graph_viz().to_string();
        if svg {
            let graph_path = temp_dir().join("output.dot");
            if fs::write(&graph_path, graph).is_err() {
                println!("cannot create temporary file");
                return ExitCode::FAILURE;
            }
            let output = Command::new("dot")
                .arg("-Tsvg")
                .arg(&graph_path)
                .output()
                .unwrap();
            String::from_utf8(output.stdout).unwrap()
        } else {
            graph
        }
    } else {
        if svg {
            println!("flag --svg must be used with flag --graph");
            return ExitCode::FAILURE;
        }
        let file = match validate_protocol_file(&file) {
            Ok(file) => file,
            Err(err) => {
                println!("{}", err);
                return ExitCode::FAILURE;
            }
        };
        let output = GenerateRust(&file).to_string();

        println!("{}", output);

        let child = Command::new("rustfmt")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        child.stdin.unwrap().write_all(output.as_bytes()).unwrap();
        let mut buffer = vec![];
        child.stdout.unwrap().read_to_end(&mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    };

    if let Some(output_path) = output_path {
        if fs::write(output_path, output).is_err() {
            println!("invalid output file");
            return ExitCode::FAILURE;
        }
    } else {
        println!("{}", output);
    }

    ExitCode::SUCCESS
}
