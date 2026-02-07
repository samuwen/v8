use log::debug;
use string_interner::symbol::SymbolU32;

use crate::{
    environment::Environment,
    global::get_string_from_pool,
    heap::{Heap, HeapId},
    lexer::Lexer,
    parser::Parser,
    span::Span,
    token::Token,
    values::{JSObject, JSResult, JSValue},
    variable::Variable,
};

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

#[allow(dead_code)]
pub struct Interpreter {
    current_environment_handle: usize,
    heap: Heap,
    source: String,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut heap = Heap::new();
        let id = heap.add_environment(Environment::new(None));
        Self {
            current_environment_handle: id,
            heap,
            source: "".to_owned(), // lil hack
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
                    println!("{}", debug_value(self, &value));
                    // let string_sym = value.to_string(self).unwrap(); // TODO - fix this later
                    // let string_value = get_string_from_pool_unchecked(&string_sym);
                    // // add quotes in
                    // if value.is_string() {
                    //     println!("'{string_value}'");
                    // } else {
                    //     println!("{string_value}");
                    // }
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

    fn add_var(&mut self, variable: Variable) -> usize {
        self.heap.add_variable(variable)
    }

    fn get_var(&mut self, var_id: usize) -> JSResult<&mut Variable> {
        self.heap.get_variable_mut(var_id)
    }

    fn new_variable(&mut self, ident_id: SymbolU32, is_mutable: bool, value: JSValue) {
        let var = Variable::new(is_mutable, value);
        let var_id = self.add_var(var);
        self.add_variable_to_current_environment(ident_id, var_id);
    }

    fn add_object(&mut self, value: JSObject) -> usize {
        self.heap.add_object(value)
    }

    fn get_object(&self, obj_id: usize) -> JSResult<&JSObject> {
        self.heap.get_object(obj_id)
    }

    fn get_object_mut(&mut self, obj_id: usize) -> JSResult<&mut JSObject> {
        self.heap.get_object_mut(obj_id)
    }

    fn add_variable_to_current_environment(&mut self, str_id: SymbolU32, var_id: usize) {
        let current_environment = self
            .get_current_environment_mut()
            .expect("Somehow you deleted all environments");
        current_environment.add_variable(str_id, var_id);
    }

    fn get_variable_from_current_environment(&self, string_id: SymbolU32) -> Option<usize> {
        let current_environment = self
            .get_current_environment()
            .expect("Somehow you deleted all environments");
        current_environment.get_variable(string_id, self)
    }

    fn get_current_environment(&self) -> JSResult<&Environment> {
        self.get_environment(self.current_environment_handle)
    }

    fn get_environment(&self, id: HeapId) -> JSResult<&Environment> {
        self.heap.get_environment(id)
    }

    fn get_environment_mut(&mut self, id: HeapId) -> JSResult<&mut Environment> {
        self.heap.get_environment_mut(id)
    }

    fn get_current_environment_mut(&mut self) -> JSResult<&mut Environment> {
        self.get_environment_mut(self.current_environment_handle)
    }

    fn add_value(&mut self, value: JSValue) -> usize {
        self.heap.add_value(value)
    }

    fn get_value(&self, id: usize) -> JSResult<&JSValue> {
        self.heap.get_value(id)
    }

    fn new_scope(&mut self) -> usize {
        let new_env = Environment::new(Some(self.current_environment_handle));
        self.heap.add_environment(new_env)
    }

    fn enter_scope(&mut self, scope_id: Option<usize>) -> usize {
        let id = match scope_id {
            Some(id) => id,
            None => self.new_scope(),
        };
        self.current_environment_handle = id;
        id
    }

    fn leave_scope(&mut self) {
        let current_env = self
            .get_current_environment_mut()
            .expect("Somehow you deleted all environments");
        current_env.expire();
        let parent = current_env.get_parent_handle();
        if parent.is_none() {
            panic!("Leave scope called on root for some reason");
        }
        let parent_id = parent.unwrap();
        self.current_environment_handle = parent_id;
    }

    fn bind_variable(&mut self, param_id: SymbolU32, value: &JSValue) -> JSResult<JSValue> {
        let var_id = self
            .get_variable_from_current_environment(param_id)
            .unwrap();
        let var = self.get_var(var_id)?;
        var.update_value(value.clone())?;
        Ok(JSValue::Undefined)
    }
}

pub fn debug_value(interpreter: &mut Interpreter, value: &JSValue) -> String {
    let out = match value {
        JSValue::Null => "null".to_string(),
        JSValue::Undefined => "undefined".to_string(),
        JSValue::Boolean { data } => data.to_string(),
        JSValue::String { data } => {
            let s = get_string_from_pool(data).unwrap_or("UNKNOWN STRING".to_string());
            format!("'{s}'")
        }
        JSValue::Symbol {
            id: _,
            description: _,
        } => todo!(),
        JSValue::Number { data } => data.to_string(),
        JSValue::BigInt => todo!(),
        JSValue::Object { object_id } => {
            let obj = interpreter.get_object(*object_id).unwrap().clone();
            obj.debug(interpreter)
        }
    };

    out
}
