use std::collections::HashMap;

use crate::{
    counter::Counter,
    environment::Environment,
    global::{get_string_from_pool, get_string_from_pool_unchecked},
    heap::Heap,
    lexer::Lexer,
    parser::Parser,
    span::Span,
    token::Token,
    values::{JSObject, JSValue},
};

mod completion_record;
mod counter;
mod environment;
mod errors;
mod expr;
mod global;
mod heap;
mod lexer;
mod parser;
mod span;
mod stmt;
mod token;
mod values;

pub struct Interpreter {
    object_heap: Heap<JSObject>,
    source: String,
    environment_heap: Heap<Environment>,
}

impl Interpreter {
    pub fn new() -> Self {
        let root_environment = Environment::new_root();
        let mut environment_heap = Heap::new();
        environment_heap.add_new_item(root_environment);
        Self {
            object_heap: Heap::new(),
            source: "".to_owned(), // lil hack
            environment_heap,
        }
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), String> {
        self.source = source.to_owned();
        let tokens = self.lex()?;

        let mut parser = Parser::new(tokens, self);
        let statements = parser.parse();

        for statement in statements {
            let res = statement.evaluate(self);
            match res {
                Ok(value) => {
                    let string_sym = value.to_string(self).unwrap(); // TODO - fix this later
                    let string_value = get_string_from_pool_unchecked(&string_sym);
                    // add quotes in
                    if value.is_string() {
                        println!("'{string_value}'");
                    } else {
                        println!("{string_value}");
                    }
                }
                Err(e) => {
                    println!("{}", e.message);
                }
            }
        }

        Ok(())
    }

    pub fn lex_only(&mut self, source: &str) -> Result<(), String> {
        self.source = source.to_owned();
        self.lex().map(|_| ())
    }

    fn lex(&mut self) -> Result<Vec<Token>, String> {
        let mut lexer = Lexer::new(&self.source);
        let tokens = lexer.lex();

        // for token in tokens.iter() {
        //     println!("{token:?}");
        // }

        if lexer.had_errors() {
            lexer.replay_errors();
            return Err(String::from("Lexer failure. Aborting"));
        }
        Ok(tokens)
    }

    fn get_source_at_span(&self, span: &Span) -> String {
        let result = &self.source[span.get_as_range()];
        result.to_string()
    }
}
