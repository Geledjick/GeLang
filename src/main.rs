mod lexer;   use lexer::*;
mod parser;  use parser::Parser;
mod emitter; use emitter::Emitter;

use std::fs;
use std::process::Command;

fn main() {
    let src = r#"
        fn fib(i32 n) i32 {
            if n <= 1 {
                return n;
            }
            fib(n - 1) + fib(n - 2)
        }
        
        fn main() {
            mut i32 i = 0;
            while i < 10 {
                println("{}", fib(i));
                i++;
            }
        }
    "#;

    let mut lexer = Lexer::new(src);
    /*
    loop {
        let tok = lexer.next_token().unwrap();
        println!("{:?}", tok);
        if tok == Token::Eof { break; }
    }
    */
    
    let program = match Parser::new(lexer).and_then(|mut p| p.parse_program()) {
        Ok(p)  => p,
        Err(e) => { eprintln!("Parse error: {e}"); return; }
    };

    
    let mut emitter = Emitter::new();
    emitter.emit_program(&program);
    let c_code = emitter.finish();

    fs::write("out.c", &c_code).unwrap();
    let status = Command::new("gcc")
        .args(["-o", "out", "out.c"])
        .status()
        .unwrap();

    if status.success() {
        println!("Compiled! Run: ./out");
    } else {
        eprintln!("gcc failed");
    }
    
}