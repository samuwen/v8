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
    let mut interpreter = Interpreter::new(args.debug);
    println!("Welcome to v8 0.0.1");

    if args.debugger {
        let source = "const x = 5;\nx;";
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
                    let res = interpreter.interpret(&line);
                    if let Err(s) = res {
                        println!("ERROR ASSHOLE {}", s);
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
