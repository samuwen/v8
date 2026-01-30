use std::collections::HashMap;

use string_interner::symbol::SymbolU32;

use crate::{
    environment::Environment,
    stmt::Stmt,
    values::{
        JSResult, JSValue, PreferredType,
        objects::ObjectProperty,
    },
};

#[derive(Clone, Debug)]
pub struct FunctionObject {
    prototype: Option<usize>,
    property_map: HashMap<SymbolU32, ObjectProperty>,
    call: Box<Stmt>, // create the statement wrapper around it before passing it thru
    environment: Environment,
    formal_parameters: Vec<SymbolU32>,
}

impl FunctionObject {
    pub fn new(call: Box<Stmt>, environment: Environment, parameters: Vec<SymbolU32>) -> Self {
        let map = HashMap::new();
        Self {
            prototype: None,
            property_map: map,
            call,
            environment,
            formal_parameters: parameters,
        }
    }

    pub fn call(&self, _this_argument: &JSValue, _arguments: Vec<&JSValue>) {
        todo!()
    }

    pub fn to_primitive(&self, hint: PreferredType) -> JSResult<JSValue> {
        todo!()
    }
}
