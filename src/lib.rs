use log::debug;
use string_interner::symbol::SymbolU32;

use crate::{
    environment::Environment,
    global::{get_string_from_pool, get_string_from_pool_unchecked},
    heap::Heap,
    lexer::Lexer,
    parser::Parser,
    span::Span,
    token::Token,
    values::{JSObject, JSValue},
    variable::Variable,
};

mod completion_record;
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
mod utils;
mod values;
mod variable;

pub struct Interpreter {
    current_environment_handle: usize,
    debug_mode: bool,
    object_heap: Heap<JSObject>,
    source: String,
    environment_heap: Heap<Environment>,
    variable_heap: Heap<Variable>,
}

impl Interpreter {
    pub fn new(debug: bool) -> Self {
        let mut environment_heap = Heap::new();
        let id = environment_heap.add_new_item(Environment::new(None));
        let variable_heap = Heap::new();
        Self {
            current_environment_handle: id,
            debug_mode: debug,
            object_heap: Heap::new(),
            source: "".to_owned(), // lil hack
            environment_heap,
            variable_heap,
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

        for token in tokens.iter() {
            debug!("{token:?}");
        }

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

    fn add_variable_to_heap(&mut self, variable: Variable) -> usize {
        self.variable_heap.add_new_item(variable)
    }

    fn get_variable_from_heap(&mut self, var_id: usize) -> &mut Variable {
        self.variable_heap.get_mut(var_id)
    }

    fn add_variable_to_current_environment(&mut self, str_id: SymbolU32, var_id: usize) {
        let mut current_environment = self.get_current_environment_mut();
        current_environment.add_variable(str_id, var_id);
    }

    fn get_variable_from_current_environment(&self, string_id: SymbolU32) -> Option<usize> {
        let current_environment = self.get_current_environment();
        current_environment.get_variable(string_id, self)
    }

    fn get_current_environment(&self) -> &Environment {
        self.environment_heap
            .get_item_from_id(self.current_environment_handle)
    }

    fn get_current_environment_mut(&mut self) -> &mut Environment {
        self.environment_heap
            .get_mut(self.current_environment_handle)
    }
}
