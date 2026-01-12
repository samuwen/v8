use std::process;

use crate::{global::SharedContext, lexer::Lexer};

mod global;
mod lexer;
mod string_pool;
mod token;

pub fn lex() {
    let source = "let x = 'hotdog';\nhotdog + (2 + 5);";
    let mut context = SharedContext::new();
    let mut lexer = Lexer::new(&mut context, source);
    lexer.lex();

    if lexer.had_errors() {
        lexer.replay_errors();
        process::exit(1);
    }

    println!("Lexing completed");
    lexer.print_tokens();
}
