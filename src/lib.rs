use log::{debug, trace, warn};
use string_interner::symbol::SymbolU32;

use crate::{
    constants::GLOBAL_THIS_NAME,
    environment::Environment,
    errors::JSError,
    global::{get_or_intern_string, get_string_from_pool},
    heap::{Heap, HeapId},
    lexer::Lexer,
    parser::Parser,
    span::Span,
    token::Token,
    values::{JSObject, JSResult, JSValue, equal, same_value},
    variable::Variable,
};

mod constants;
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
    heap: Heap,
    object_proto_id: usize,
    function_proto_id: usize,
    output_buffer: String,
    error_buffer: String,
    source: String,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut heap = Heap::new();
        let object_proto = JSObject::create_object_proto(); // should always be 0. store anyways
        let proto_id = heap.add_object(object_proto);
        let env_id = heap.add_environment(Environment::new(None));
        let function_proto = JSObject::create_function_proto(env_id, proto_id);
        let function_proto_id = heap.add_object(function_proto);
        Self {
            current_environment_handle: env_id,
            heap,
            object_proto_id: proto_id,
            function_proto_id,
            output_buffer: String::new(),
            error_buffer: String::new(),
            source: "".to_owned(), // lil hack
        }
    }

    pub fn setup(mut self) -> Self {
        JSObject::create_global_object(&mut self);
        trace!("{}", self.heap);
        self
    }

    pub fn interpret(&mut self, source: &str) -> Result<(String, String), String> {
        self.source = source.to_owned();
        let tokens = self.lex()?;

        let mut parser = Parser::new(tokens, self);
        let statements = parser.parse();

        for statement in statements {
            debug!("raw_statement: {statement}");
            let res = statement.evaluate(self);
            match res {
                Ok(value) => {
                    debug!("debug_value: {}", debug_value(self, &value));
                }
                Err(e) => {
                    eprintln!("{}", e.message);
                }
            }
        }

        let out = self.output_buffer.clone();
        let err = self.error_buffer.clone();

        Ok((out, err))
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

    fn get_value_from_environment(
        &mut self,
        env_id_opt: Option<usize>,
        str_id: SymbolU32,
    ) -> JSResult<&JSValue> {
        let env_id = env_id_opt.unwrap_or(self.current_environment_handle);
        let environment = self.get_environment(env_id)?;
        let var_result = environment.get_variable(str_id, self);
        match var_result {
            Some(var_id) => {
                let var = self.get_var(var_id)?;
                let val = var.get_value();
                return Ok(val);
            }
            None => {
                let parent_handle_opt = environment.get_parent_handle();
                match parent_handle_opt {
                    Some(handle) => {
                        return self.get_value_from_environment(Some(handle), str_id);
                    }
                    None => {
                        let global_this_id = get_or_intern_string(GLOBAL_THIS_NAME);
                        let global_this_val =
                            self.get_value_from_environment(None, global_this_id)?;
                        let object_id = global_this_val.get_object_id()?;
                        let global_this = self.get_object(object_id)?;
                        let prop = global_this.get_property(&str_id);
                        if let Some(prop) = prop {
                            let val = prop.get_value()?;
                            return Ok(val);
                        }
                    }
                }
            }
        }

        Err(JSError::new("Variable not found in environment"))
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
        let parent = current_env.get_parent_handle();
        if parent.is_none() {
            // we're in root, likely a global scoped fn was called
            warn!("Attempting to leave the global scope. Programmer error?");
            return;
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
        debug!("var: {var:?}");
        Ok(JSValue::Undefined)
    }

    fn get_object_proto_id(&self) -> usize {
        self.object_proto_id
    }

    fn same_type(&self, left: &JSValue, right: &JSValue) -> JSResult<JSValue> {
        Ok(JSValue::new_boolean(match left {
            JSValue::Null => match right {
                JSValue::Null => true,
                _ => false,
            },
            JSValue::Undefined => match right {
                JSValue::Undefined => true,
                _ => false,
            },
            JSValue::Boolean { data: _ } => match right {
                JSValue::Boolean { data: _ } => true,
                _ => false,
            },
            JSValue::String { data: _ } => match right {
                JSValue::String { data: _ } => true,
                _ => false,
            },
            JSValue::Symbol {
                id: _,
                description: _,
            } => match right {
                JSValue::Symbol {
                    id: _,
                    description: _,
                } => true,
                _ => false,
            },
            JSValue::Number { data: _ } => match right {
                JSValue::Number { data: _ } => true,
                _ => false,
            },
            JSValue::BigInt => match right {
                JSValue::BigInt => true,
                _ => false,
            },
            JSValue::Object {
                object_id: _,
                kind: _,
            } => match right {
                JSValue::Object {
                    object_id: _,
                    kind: _,
                } => true,
                _ => false,
            },
        }))
    }

    fn same_value(&mut self, left: &JSValue, right: &JSValue) -> JSResult<JSValue> {
        match left {
            JSValue::Number { data: l_num } => {
                let r_num = right.to_number(self)?.get_number();
                return Ok(JSValue::new_boolean(same_value(*l_num, r_num)));
            }
            _ => self.same_value_non_number(left, right),
        }
    }

    fn same_value_non_number(&mut self, left: &JSValue, right: &JSValue) -> JSResult<JSValue> {
        Ok(JSValue::new_boolean(match left {
            JSValue::Null | JSValue::Undefined => true,
            JSValue::Boolean { data } => {
                let right = right.to_boolean();
                let result = *data == right;
                result
            }
            JSValue::String { data } => {
                let right = right.to_string(self)?;
                *data == right
            }
            _ => true,
        }))
    }

    fn is_loosely_equal(&mut self, left: &JSValue, right: &JSValue) -> JSResult<JSValue> {
        let is_same_type = self.same_type(left, right)?.get_boolean();
        if is_same_type {
            return self.is_strictly_equal(left, right);
        }
        if left.is_null() && right.is_undefined() {
            return Ok(JSValue::new_boolean(true));
        }
        if left.is_undefined() && right.is_null() {
            return Ok(JSValue::new_boolean(true));
        }
        if left.is_number() && right.is_string() {
            let right = right.to_number(self)?;
            return self.is_loosely_equal(left, &right);
        }

        if left.is_string() && right.is_number() {
            let left = left.to_number(self)?;
            return self.is_loosely_equal(&left, right);
        }

        if left.is_big_int() && right.is_string() {
            todo!()
        }

        if left.is_string() && right.is_big_int() {
            return self.is_loosely_equal(right, left);
        }

        if left.is_boolean() {
            let left = left.to_number(self)?;
            return self.is_loosely_equal(&left, right);
        }

        if right.is_boolean() {
            let right = right.to_number(self)?;
            return self.is_loosely_equal(left, &right);
        }

        if (left.is_string() || left.is_number() || left.is_big_int() || left.is_symbol())
            && right.is_object()
        {
            let right = right.to_primitive(None, self)?;
            return self.is_loosely_equal(left, &right);
        }

        if (right.is_string() || right.is_number() || right.is_big_int() || right.is_symbol())
            && left.is_object()
        {
            let left = right.to_primitive(None, self)?;
            return self.is_loosely_equal(&left, right);
        }

        Ok(JSValue::new_boolean(false))
    }

    fn is_strictly_equal(&mut self, left: &JSValue, right: &JSValue) -> JSResult<JSValue> {
        let is_same_type = self.same_type(left, right)?.get_boolean();
        if !is_same_type {
            return Ok(JSValue::new_boolean(false));
        }
        if left.is_number() {
            let res = equal(left.get_number(), right.get_number());
            return Ok(JSValue::new_boolean(res));
        }
        return self.same_value_non_number(left, right);
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
        JSValue::Object { object_id, kind: _ } => {
            let obj = interpreter.get_object(*object_id).unwrap().clone();
            obj.debug(interpreter)
        }
    };

    out
}
