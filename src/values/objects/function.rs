use std::collections::HashMap;

use log::debug;
use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    errors::ErrorKind,
    stmt::Stmt,
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
    pub fn new(call: Box<Stmt>, environment_id: usize, parameters: Vec<SymbolU32>) -> Self {
        let map = HashMap::new();
        Self {
            prototype: None,
            properties: map,
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
        debug!("{:?}", arguments);
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
}
