use std::{
    env::{self, temp_dir},
    fs,
    process::{Command, ExitCode},
};

use obbidl_lib::{
    ast::File, compile::compile_protocol_file, format::binary::Binary, format_rust,
    generate::GenerateRust, parser::parse, validate::validate_protocol_file,
};

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
    let file = match parse::<File>(&source) {
        Ok(ast) => ast,
        Err(err) => {
            println!("{}", err);
            return ExitCode::FAILURE;
        }
    };
    let file_fsm = compile_protocol_file(&file);

    let output = if graph {
        let graph = file_fsm.graph_viz().to_string();
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
        let file = match validate_protocol_file(&file_fsm, &file.structs) {
            Ok(file) => file,
            Err(errors) => {
                for err in errors {
                    println!("{}", err.pretty_print(&source));
                }
                return ExitCode::FAILURE;
            }
        };
        let output = GenerateRust::<Binary>::new(&file).to_string();

        format_rust(&output)
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
