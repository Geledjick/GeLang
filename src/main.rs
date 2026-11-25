mod lexer;   use lexer::*;
mod parser;  use parser::Parser;
mod emitter; use emitter::Emitter;
mod typechecker; use typechecker::TypeChecker;

use std::{fs, io};
use std::io::ErrorKind;
use std::process::{Command, ExitCode};
use std::env;

struct Config<'a> {
    inputs: Vec<&'a str>,
    out: &'a str,
    debug: bool,
}

fn parse_args<'a>(args: &'a [String]) -> io::Result<Config<'a>> {
    let mut inputs: Vec<&'a str> = Vec::with_capacity(8);
    let mut out_path: Option<&'a str> = None;
    let mut debug: bool = false;
    
    let mut iter = args.iter().skip(1);
    
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-o" => {
                let out = iter.next().ok_or_else(|| {
                    io::Error::new(ErrorKind::InvalidInput, "Expected output path after '-o'")
                })?;
                out_path = Some(out.as_str());
            }

            "--debug" => debug = true,

            flag if flag.starts_with('-') => {
                return Err(
                    io::Error::new(ErrorKind::InvalidInput, format!("Unknown flag: {}", flag))
                );
            }

            path => inputs.push(path),
        }
    }

    if inputs.is_empty() {
        return Err(
            io::Error::new(ErrorKind::InvalidInput, "Usage: gelang <src...> [-o <out>] [-v]"),
        );
    }
    
    let out_path = out_path.unwrap_or("out");
    Ok(Config { inputs, out: out_path, debug })
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    let config = match parse_args(&args) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("Error: {}", err);
            return ExitCode::FAILURE;
        }
    };

    let mut src = String::new();
    for path in &config.inputs {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(err) => {
                eprintln!("Failed to read '{}': {}", path, err);
                return ExitCode::FAILURE;
            }
        };
    
        src.push_str(&content);

        if !src.ends_with('\n') {
            src.push('\n');
        }
    }
    
    let lexer = Lexer::new(&src);
    let mut program = match Parser::new(lexer).and_then(|mut p| p.parse_program()) {
        Ok(p) => {
            if config.debug {
                println!("{:#?}", p);
            }
            p
        },
        Err(e) => {
            eprintln!("Parse error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut checker = TypeChecker::new();
    match checker.check_program(&mut program) {
        Ok(())  => {}
        Err(e)  => {
            eprintln!("Type error: {e}");
            return ExitCode::FAILURE;
        }
    }

    let mut emitter = Emitter::new();
    emitter.emit_program(&program);
    let c_code = emitter.finish();

    let out_c = format!("{}.c", config.out);
    fs::write(&out_c, &c_code).unwrap();
    let status = Command::new("gcc")
        .args(["-o", &config.out, &out_c])
        .status()
        .unwrap();

    if status.success() {
        println!("Compiled! Run: ./{}", config.out);
    } else {
        eprintln!("gcc failed");
    }

    ExitCode::SUCCESS
}