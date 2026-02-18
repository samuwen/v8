#![allow(dead_code)]
#![allow(unused_variables)]

use std::fs::File;
use std::io::Read;

use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use v8::Interpreter;

use crate::utils::Args;

mod utils;

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    let mut rl = DefaultEditor::new()?;
    let mut ctrl_c_once = false;
    let mut interpreter = Interpreter::new().setup();

    // we're in file land, don't need the repl
    if args.path.is_some() {
        let path = args.path.unwrap();
        let extension = path.extension();
        match extension {
            Some(ext) => {
                if ext != "js" {
                    // we dunno what this is so just fail out
                    std::process::exit(1)
                }
            }
            None => {
                // we dunno what this is so just fail out
                std::process::exit(1)
            }
        }
        let mut file =
            File::open(&path).unwrap_or_else(|_| panic!("Cannot find module {:?}", &path));
        let mut source = String::new();
        let res = file.read_to_string(&mut source);
        if res.is_err() {
            // we dunno what this is so just fail out
            std::process::exit(1);
        }
        // we have a valid js file that's been read into a string
        let (out, err) = interpreter.interpret(&source).unwrap();
        if out.len() > 0 {
            println!("{out}");
        }
        if err.len() > 0 {
            eprintln!("{err}");
        }
        std::process::exit(0);
    }
    println!("Welcome to v8 0.0.1");

    if args.debugger {
        let source = "let x = 5;\nx = 6;";
        interpreter.interpret(source).unwrap();
    } else {
        'repl: loop {
            let readline = rl.readline("> ");
            match readline {
                Ok(line) => {
                    if line == ".exit" || line == "exit()" {
                        break 'repl;
                    }
                    let line = if !line.ends_with(';') {
                        format!("{line};")
                    } else {
                        line
                    };
                    let (out, err) = interpreter.interpret(&line).unwrap();
                    if out.len() > 0 {
                        print!("{out}");
                    }
                    if err.len() > 0 {
                        eprint!("{err}");
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    if ctrl_c_once {
                        break 'repl;
                    } else {
                        println!("(To exit, press Ctrl+C again or Ctrl+D or type .exit)");
                        ctrl_c_once = true;
                    }
                }
                Err(ReadlineError::Eof) => {
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
    }
    Ok(())
}
