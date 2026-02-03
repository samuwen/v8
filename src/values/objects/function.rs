#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashMap;

use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    stmt::Stmt,
    values::{JSResult, JSValue, PreferredType, objects::ObjectProperty},
};

#[derive(Clone, Debug)]
pub struct FunctionObject {
    prototype: Option<usize>,
    property_map: HashMap<SymbolU32, ObjectProperty>,
    call: Box<Stmt>, // create the statement wrapper around it before passing it thru
    environment_id: usize,
    formal_parameters: Vec<SymbolU32>,
}

impl FunctionObject {
    pub fn new(call: Box<Stmt>, environment_id: usize, parameters: Vec<SymbolU32>) -> Self {
        let map = HashMap::new();
        Self {
            prototype: None,
            property_map: map,
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
        let args = arguments.iter();
        for (param, arg) in self.formal_parameters.iter().zip(args) {
            interpreter.bind_variable(*param, arg)?;
        }
        let result = self.call.evaluate(interpreter)?;
        interpreter.leave_scope();
        Ok(result)
    }

    pub fn to_primitive(&self, hint: PreferredType) -> JSResult<JSValue> {
        todo!()
    }
}
