use std::env;
use std::fs;
use std::io;

use rlox::prelude::*;

/// The conventional exit code in BSD Unixes.
/// See: man 3 sysexits
mod ex {
    /// The conventional exit code for usage error.
    pub const USAGE: i32 = 64;
    /// When the input data is incorrect -- for example, a compile-time error.
    pub const DATAERR: i32 = 65;
    /// An internal software error occured.
    pub const SOFTWARE: i32 = 70;
    /// An error occured while doing I/O on a file.
    pub const IOERR: i32 = 74;
}

fn main() -> rlox::Result<()> {
    let args: Vec<_> = env::args().collect();

    if args.len() <= 1 {
        repl()
    } else if args.len() == 2 {
        let filename = args.get(1).unwrap();
        run_file(filename)
    } else {
        eprintln!("Usage: rlox [path]");
        std::process::exit(ex::USAGE);
    }
}

/// Use Lox interactively using the read-execute-print loop.
fn repl() -> rlox::Result<()> {
    let mut vm = VM::default();
    let mut line = String::with_capacity(1024);

    let stdin = io::stdin();

    loop {
        line.clear();

        print!("> ");
        match stdin.read_line(&mut line) {
            Ok(_) => {
                vm.interpret(&line)?;
            }
            Err(_) => {
                println!();
                break;
            }
        }
    }

    Ok(())
}

fn run_file(filename: &str) -> rlox::Result<()> {
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Could not read file: {filename}");
            std::process::exit(ex::IOERR);
        }
    };
    let mut vm = VM::default();

    use InterpretationError::*;
    let status = match vm.interpret(&source) {
        Ok(_) => 0,
        Err(CompileError) => ex::DATAERR,
        Err(RuntimeError) => ex::SOFTWARE,
    };

    std::process::exit(status)
}
