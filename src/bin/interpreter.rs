use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use v8::Interpreter;

fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    let mut ctrl_c_once = false;
    let mut interpreter = Interpreter::new();
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
    Ok(())
}
