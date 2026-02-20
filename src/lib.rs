use log::{debug, info, trace};
use string_interner::{Symbol, symbol::SymbolU32};

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
    environment_stack: Vec<usize>,
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
        let env_id = heap.add_environment(Environment::new());
        let function_proto = JSObject::create_function_proto(env_id, proto_id);
        let function_proto_id = heap.add_object(function_proto);
        let environment_stack = vec![env_id];
        Self {
            environment_stack,
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

        debug!("=========== LEXER OUTPUT ===========");
        for token in tokens.iter() {
            debug!("{token:?}");
        }
        debug!("=========== END LEXER OUTPUT ===========\n");

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

    fn get_object(&self, object_id: usize) -> JSResult<&JSObject> {
        self.heap.get_object(object_id)
    }

    fn get_object_mut(&mut self, obj_id: usize) -> JSResult<&mut JSObject> {
        self.heap.get_object_mut(obj_id)
    }

    fn add_variable_to_current_environment(&mut self, str_id: SymbolU32, var_id: usize) {
        let current_env_handle = self.get_current_environment_handle().clone();
        let current_environment = self
            .get_current_environment_mut()
            .expect("Somehow you deleted all environments");
        current_environment.add_variable(str_id, var_id);
        debug!(
            "Added variable id: {} to environment ({current_env_handle}) {current_environment}",
            str_id.to_usize()
        );
    }

    fn get_value_from_environment(&mut self, str_id: SymbolU32) -> JSResult<&JSValue> {
        for id in self.environment_stack.iter().rev() {
            let environment = self.get_environment(*id)?;
            let var_result = environment.get_variable(str_id);
            if let Some(var_id) = var_result {
                let var = self.get_var(var_id)?;
                let val = var.get_value();
                return Ok(val);
            }
        }

        // we didn't find the variable - so check the global object since it wasn't invoked directly
        self.get_value_from_global_this(str_id)
    }

    fn does_local_environment_already_have_variable(&self, string_id: &SymbolU32) -> bool {
        let environment_handle = self
            .environment_stack
            .last()
            .expect("Environment stack empty, something is fucked");
        let environment = self
            .get_environment(*environment_handle)
            .expect("Environment ID not found in heap");
        environment.has_variable(string_id)
    }

    fn get_value_from_global_this(&mut self, str_id: SymbolU32) -> JSResult<&JSValue> {
        let global_environment_id = self
            .environment_stack
            .get(0)
            .expect("Why did you delete the global environment?"); // should always exist
        let global_environment = self.get_environment(*global_environment_id)?;
        let global_this = get_or_intern_string(GLOBAL_THIS_NAME);
        let var_result = global_environment.get_variable(global_this);
        if let Some(var_id) = var_result {
            let var = self.get_var(var_id)?;
            let val = var.get_value().clone();
            // always true
            if let JSValue::Object { object_id, kind: _ } = val {
                let object = self.get_object(object_id)?;
                let prop = object.get_property(&str_id);
                if let Some(prop) = prop {
                    let value = prop.get_value()?;
                    return Ok(value);
                }
            }
        }
        Err(JSError::new("Variable not found in environment"))
    }

    fn get_variable_from_current_environment(
        &mut self,
        string_id: SymbolU32,
    ) -> JSResult<&mut Variable> {
        for id in self.environment_stack.iter().rev() {
            let environment = self.get_environment(*id)?;
            let var_result = environment.get_variable(string_id);
            if let Some(var_id) = var_result {
                let var = self.get_var(var_id)?;
                return Ok(var);
            }
        }
        Err(JSError::new("Variable not found"))
    }

    fn get_environment(&self, id: HeapId) -> JSResult<&Environment> {
        self.heap.get_environment(id)
    }

    fn get_environment_mut(&mut self, id: HeapId) -> JSResult<&mut Environment> {
        self.heap.get_environment_mut(id)
    }

    fn get_current_environment_handle(&self) -> usize {
        *self
            .environment_stack
            .last()
            .expect("Environment stack is empty")
    }

    fn get_current_environment_mut(&mut self) -> JSResult<&mut Environment> {
        let handle = self.get_current_environment_handle();
        self.get_environment_mut(handle)
    }

    fn add_value(&mut self, value: JSValue) -> usize {
        self.heap.add_value(value)
    }

    fn get_value(&self, id: usize) -> JSResult<&JSValue> {
        self.heap.get_value(id)
    }

    fn new_scope(&mut self) -> usize {
        let new_env = Environment::new();
        self.heap.add_environment(new_env)
    }

    fn enter_scope(&mut self, scope_id: Option<usize>) -> usize {
        info!("Entering scope");
        let id = match scope_id {
            Some(id) => id,
            None => self.new_scope(),
        };
        self.environment_stack.push(id);
        id
    }

    fn leave_scope(&mut self) {
        info!("Leaving scope");
        self.environment_stack.pop();
    }

    fn bind_variable(&mut self, param_id: SymbolU32, value: &JSValue) -> JSResult<JSValue> {
        let var = self
            .get_variable_from_current_environment(param_id)
            .unwrap();
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

    fn _same_value(&mut self, left: &JSValue, right: &JSValue) -> JSResult<JSValue> {
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
