#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashMap;

use crate::{
    environment::Environment,
    errors::JSError,
    values::{JSObject, JSResult, JSValue},
    variable::Variable,
};

pub type HeapId = usize;

#[derive(Debug)]
enum HeapValue {
    Environment(Environment),
    Variable(Variable),
    Value(JSValue),
    Object(JSObject),
}

impl HeapValue {
    pub fn new_environment(env: Environment) -> Self {
        Self::Environment(env)
    }

    pub fn new_variable(var: Variable) -> Self {
        Self::Variable(var)
    }

    pub fn new_value(val: JSValue) -> Self {
        Self::Value(val)
    }

    pub fn new_object(obj: JSObject) -> Self {
        Self::Object(obj)
    }
}

impl std::fmt::Display for HeapValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeapValue::Environment(environment) => write!(f, "{environment}"),
            HeapValue::Variable(variable) => write!(f, "{variable}"),
            HeapValue::Value(jsvalue) => write!(f, "{jsvalue:?}"),
            HeapValue::Object(jsobject) => write!(f, "{jsobject:?}"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Heap {
    map: HashMap<HeapId, HeapValue>,
    counter: HeapId,
}

impl Heap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_environment(&mut self, env: Environment) -> HeapId {
        let value = HeapValue::new_environment(env);
        self.add_to_map(value)
    }

    pub fn add_variable(&mut self, var: Variable) -> HeapId {
        let value = HeapValue::new_variable(var);
        self.add_to_map(value)
    }

    pub fn add_value(&mut self, val: JSValue) -> HeapId {
        let value = HeapValue::new_value(val);
        self.add_to_map(value)
    }

    pub fn add_object(&mut self, obj: JSObject) -> HeapId {
        let value = HeapValue::new_object(obj);
        self.add_to_map(value)
    }

    pub fn get_environment(&self, id: HeapId) -> JSResult<&Environment> {
        let heap_val_opt = self.get(id);
        match heap_val_opt {
            Some(hv) => match hv {
                HeapValue::Environment(env) => Ok(env),
                _ => Err(JSError::new_not_found("Environment", id)),
            },
            _ => Err(JSError::new_not_found("Environment", id)),
        }
    }

    pub fn get_environment_mut(&mut self, id: HeapId) -> JSResult<&mut Environment> {
        let heap_val_opt = self.get_mut(id);
        match heap_val_opt {
            Some(hv) => match hv {
                HeapValue::Environment(env) => Ok(env),
                _ => Err(JSError::new_not_found("Environment", id)),
            },
            _ => Err(JSError::new_not_found("Environment", id)),
        }
    }

    pub fn get_value(&self, id: HeapId) -> JSResult<&JSValue> {
        let heap_val_opt = self.get(id);
        match heap_val_opt {
            Some(hv) => match hv {
                HeapValue::Value(val) => Ok(val),
                _ => Err(JSError::new_not_found("Value", id)),
            },
            _ => Err(JSError::new_not_found("Value", id)),
        }
    }

    pub fn get_value_mut(&mut self, id: HeapId) -> JSResult<&mut JSValue> {
        let heap_val_opt = self.get_mut(id);
        match heap_val_opt {
            Some(hv) => match hv {
                HeapValue::Value(val) => Ok(val),
                _ => Err(JSError::new_not_found("Value", id)),
            },
            _ => Err(JSError::new_not_found("Value", id)),
        }
    }

    pub fn get_variable(&self, id: HeapId) -> JSResult<&Variable> {
        let heap_val_opt = self.get(id);
        match heap_val_opt {
            Some(hv) => match hv {
                HeapValue::Variable(var) => Ok(var),
                _ => Err(JSError::new_not_found("Variable", id)),
            },
            _ => Err(JSError::new_not_found("Variable", id)),
        }
    }

    pub fn get_variable_mut(&mut self, id: HeapId) -> JSResult<&mut Variable> {
        let heap_val_opt = self.get_mut(id);
        match heap_val_opt {
            Some(hv) => match hv {
                HeapValue::Variable(var) => Ok(var),
                _ => Err(JSError::new_not_found("Variable", id)),
            },
            _ => Err(JSError::new_not_found("Variable", id)),
        }
    }

    pub fn get_object(&self, id: HeapId) -> JSResult<&JSObject> {
        let heap_val_opt = self.get(id);
        match heap_val_opt {
            Some(hv) => match hv {
                HeapValue::Object(obj) => Ok(obj),
                _ => Err(JSError::new_not_found("Object", id)),
            },
            _ => Err(JSError::new_not_found("Object", id)),
        }
    }

    pub fn get_object_mut(&mut self, id: HeapId) -> JSResult<&mut JSObject> {
        let heap_val_opt = self.get_mut(id);
        match heap_val_opt {
            Some(hv) => match hv {
                HeapValue::Object(obj) => Ok(obj),
                _ => Err(JSError::new_not_found("Object", id)),
            },
            _ => Err(JSError::new_not_found("Object", id)),
        }
    }

    fn get(&self, id: HeapId) -> Option<&HeapValue> {
        self.map.get(&id)
    }

    fn get_mut(&mut self, id: HeapId) -> Option<&mut HeapValue> {
        self.map.get_mut(&id)
    }

    fn add_to_map(&mut self, value: HeapValue) -> HeapId {
        let id = self.get_next_id();
        self.map.insert(id, value);
        id
    }

    fn get_next_id(&mut self) -> HeapId {
        let id = self.counter;
        self.counter += 1;
        id
    }
}

impl std::fmt::Display for Heap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Heap: {{")?;
        for (id, value) in &self.map {
            writeln!(f, "{id}: {value}")?;
        }
        writeln!(f, "}}")
    }
}
