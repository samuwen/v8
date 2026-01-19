use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use v8::Interpreter;

use crate::utils::Args;

mod utils;

fn main() -> Result<()> {
    let args = Args::parse();
    let mut rl = DefaultEditor::new()?;
    let mut ctrl_c_once = false;
    let mut interpreter = Interpreter::new();
    if args.debug {
        let source = "2 + 2 + 2;";
        interpreter.interpret(source).unwrap();
    } else {
        'repl: loop {
            let readline = rl.readline("> ");
            match readline {
                Ok(line) => {
                    if line == ".exit" || line == "exit()" {
                        break 'repl;
                    }
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
