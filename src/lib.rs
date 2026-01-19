use crate::{
    global::set_source_string, heap::Heap, lexer::Lexer, parser::Parser, token::Token,
    values::JSObject,
};

mod completion_record;
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
    heap: Heap,
    source: String,
}

impl Interpreter {
    pub fn new(source: &str) -> Self {
        Self {
            heap: Heap::new(),
            source: source.to_string(),
        }
    }

    pub fn interpret(&mut self) -> Result<(), String> {
        let tokens = self.lex(&self.source.clone())?;

        let mut parser = Parser::new(tokens);
        let statements = parser.parse();

        for statement in statements {
            let value = statement.evaluate(self);
            println!("{value:?}");
        }

        Ok(())
    }

    pub fn lex_only(&mut self, source: &str) -> Result<(), String> {
        self.lex(source).map(|_| ())
    }

    fn lex(&mut self, source: &str) -> Result<Vec<Token>, String> {
        set_source_string(source);
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex();

        for token in tokens.iter() {
            println!("{token:?}");
        }

        if lexer.had_errors() {
            lexer.replay_errors();
            return Err(String::from("Lexer failure. Aborting"));
        }
        Ok(tokens)
    }
}
