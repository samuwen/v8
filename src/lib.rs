use std::process;

use crate::{emitter::Emitter, global::SharedContext, lexer::Lexer, parser::Parser};

mod emitter;
mod expr;
mod global;
mod lexer;
mod parser;
mod stmt;
mod token;
mod value;

pub struct Interpreter {
    context: SharedContext,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            context: SharedContext::new(),
        }
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), String> {
        let mut lexer = Lexer::new(&mut self.context, source);
        let tokens = lexer.lex();

        if lexer.had_errors() {
            lexer.replay_errors();
            return Err(String::from("Lexer failure. Aborting"));
        }

        let mut parser = Parser::new(tokens);
        let statements = parser.parse();

        let mut emitter = Emitter::new(&mut self.context, statements);
        let value = emitter.evaluate();
        println!("{value}");

        Ok(())
    }
}

pub fn lex() {
    let source = "const z = (x, y) => x + y;";
    let mut context = SharedContext::new();
    let mut lexer = Lexer::new(&mut context, source);
    let tokens = lexer.lex();

    if lexer.had_errors() {
        lexer.replay_errors();
        process::exit(1);
    }

    println!("Lexing completed");
    // lexer.print_tokens();

    let mut parser = Parser::new(tokens);
    let statements = parser.parse();

    for statement in statements {
        println!("{statement}");
    }
}
