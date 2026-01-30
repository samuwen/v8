use env_logger::Env;
use log::debug;
use string_interner::symbol::SymbolU32;

use crate::{
    environment::Environment,
    global::get_string_from_pool_unchecked,
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
    object_heap: Heap<JSObject>,
    source: String,
    environment_heap: Heap<Environment>,
    variable_heap: Heap<Variable>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut environment_heap = Heap::new();
        let id = environment_heap.add_new_item(Environment::new(None));
        let variable_heap = Heap::new();
        Self {
            current_environment_handle: id,
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

    fn new_variable(&mut self, ident_id: SymbolU32, is_mutable: bool, value: JSValue) {
        let var = Variable::new(is_mutable, value);
        let var_id = self.add_variable_to_heap(var);
        self.add_variable_to_current_environment(ident_id, var_id);
    }

    fn add_variable_to_current_environment(&mut self, str_id: SymbolU32, var_id: usize) {
        let current_environment = self.get_current_environment_mut();
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

    fn enter_scope(&mut self) {
        let new_env = Environment::new(Some(self.current_environment_handle));
        let id = self.environment_heap.add_new_item(new_env);
        self.current_environment_handle = id;
    }

    fn leave_scope(&mut self) {
        let current_env = self.get_current_environment_mut();
        current_env.expire();
        let parent = current_env.get_parent_handle();
        if parent.is_none() {
            panic!("Leave scope called on root for some reason");
        }
        let parent_id = parent.unwrap();
        self.current_environment_handle = parent_id;
    }
}
