use core::f64;
use std::collections::HashMap;

use log::debug;
use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    errors::ErrorKind,
    expr::{Expr, LogKind},
    global::{get_or_intern_string, get_string_from_pool},
    stmt::Stmt,
    token::Kind,
    values::{JSResult, JSValue, PreferredType, objects::ObjectProperty},
};

#[derive(Clone, Debug)]
pub struct FunctionObject {
    prototype: Option<usize>,
    properties: HashMap<SymbolU32, ObjectProperty>,
    call: Box<Stmt>,
    environment_id: usize,
    formal_parameters: Vec<SymbolU32>,
}

impl FunctionObject {
    pub fn new_proto(env_id: usize, proto_id: usize) -> Self {
        let mut properties = HashMap::new();
        let length_id = get_or_intern_string("length");
        let name_id = get_or_intern_string("name");
        let length_val = ObjectProperty::new_from_value(JSValue::new_number(&0.0));
        properties.insert(length_id, length_val);
        let name_string_id = get_or_intern_string("");
        let name_val = ObjectProperty::new_from_value(JSValue::new_string(&name_string_id));
        properties.insert(name_id, name_val);
        Self {
            prototype: Some(proto_id),
            environment_id: env_id,
            call: Box::new(Stmt::Break),
            formal_parameters: vec![],
            properties,
        }
    }

    pub fn new(
        call: Box<Stmt>,
        environment_id: usize,
        proto_id: usize,
        parameters: Vec<SymbolU32>,
    ) -> Self {
        Self {
            prototype: Some(proto_id),
            properties: HashMap::new(),
            call,
            environment_id,
            formal_parameters: parameters,
        }
    }

    pub fn call(
        &self,
        arguments: Vec<JSValue>,
        interpreter: &mut Interpreter,
    ) -> JSResult<JSValue> {
        interpreter.enter_scope(Some(self.environment_id));
        debug!("function arguments: {:?}", arguments);
        let args = arguments.iter();
        for (param, arg) in self.formal_parameters.iter().zip(args) {
            interpreter.bind_variable(*param, arg)?;
        }
        let result = self.call.evaluate(interpreter);
        let result = match result {
            Ok(v) => v,
            Err(e) => match e.kind {
                ErrorKind::Return(id) => {
                    let value = interpreter.get_value(id)?;
                    value.clone()
                }
                _ => {
                    return Err(e);
                }
            },
        };
        interpreter.leave_scope();
        Ok(result)
    }

    pub fn to_primitive(&self, hint: PreferredType) -> JSResult<JSValue> {
        todo!()
    }

    pub fn debug(&self, interpreter: &mut Interpreter) -> String {
        format!("Function! idk whats in it")
    }

    pub fn create_is_finite(interpreter: &mut Interpreter) -> Self {
        let is_finite_arg_id = get_or_intern_string(&format!("is_finite_arg"));
        let left = Expr::new_identifier(&is_finite_arg_id);
        let right = Expr::new_literal(JSValue::Number {
            data: f64::INFINITY,
        });
        let binary_expr = Expr::new_binary(Kind::NotEqual, left, right);
        let stmt = Stmt::new_expression(binary_expr);
        Self::new_built_in(is_finite_arg_id, stmt, interpreter)
    }

    pub fn create_log(interpreter: &mut Interpreter) -> Self {
        Self::create_generic_logger(LogKind::Log, interpreter)
    }

    pub fn create_error(interpreter: &mut Interpreter) -> Self {
        Self::create_generic_logger(LogKind::Error, interpreter)
    }

    fn create_generic_logger(kind: LogKind, interpreter: &mut Interpreter) -> Self {
        let log_id = get_or_intern_string("data");
        let print_expr = Expr::new_print_expr(kind);
        let stmt = Stmt::new_expression(print_expr);
        Self::new_built_in(log_id, stmt, interpreter)
    }

    pub fn new_built_in(arg_id: SymbolU32, stmt: Stmt, interpreter: &mut Interpreter) -> Self {
        let scope_id = interpreter.enter_scope(None);
        let parameters = vec![arg_id];
        for param in &parameters {
            interpreter.new_variable(arg_id, true, JSValue::Undefined);
        }
        interpreter.leave_scope();

        Self {
            call: Box::new(stmt),
            prototype: None,
            properties: HashMap::new(),
            environment_id: scope_id,
            formal_parameters: parameters,
        }
    }
}

impl std::fmt::Display for FunctionObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Function: {{")?;
        for (key, value) in &self.properties {
            let string = get_string_from_pool(&key).unwrap();
            writeln!(f, "\t {string}: {:?}", value.get_value())?;
        }
        writeln!(f, "}}")
    }
}
