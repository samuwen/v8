use std::process;

use crate::{global::SharedContext, lexer::Lexer, parser::Parser};

mod expr;
mod global;
mod lexer;
mod parser;
mod stmt;
mod token;

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
