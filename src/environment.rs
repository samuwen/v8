#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashMap;

use log::trace;
use string_interner::symbol::SymbolU32;

use crate::Interpreter;

type EnvironmentId = usize;
type StringId = SymbolU32;

#[derive(Debug, Clone)]
pub struct Environment {
    is_expired: bool,
    handles: HashMap<StringId, usize>, // stringID: variableID (maps string names to variable ids)
    parent_id: Option<EnvironmentId>,
}

impl Environment {
    pub fn new(parent_id: Option<EnvironmentId>) -> Self {
        Self {
            is_expired: false,
            parent_id,
            handles: HashMap::new(),
        }
    }

    pub fn has_variable(&self, string_id: StringId, interpreter: &Interpreter) -> bool {
        if self.handles.contains_key(&string_id) {
            return true;
        }
        if let Some(parent) = self.parent_id {
            let parent_env = interpreter.get_environment(parent);
            if parent_env.is_err() {
                return false;
            }
            let parent_env = parent_env.unwrap();
            return parent_env.has_variable(string_id, interpreter);
        }
        false
    }

    pub fn get_variable(&self, string_id: StringId, interpreter: &Interpreter) -> Option<usize> {
        let maybe_handle = self.handles.get(&string_id);
        if let Some(handle) = maybe_handle {
            return Some(*handle);
        }
        if let Some(parent) = self.parent_id {
            let parent_env = interpreter.get_environment(parent).ok()?;
            return parent_env.get_variable(string_id, interpreter);
        }
        None
    }

    pub fn add_variable(&mut self, string_id: StringId, variable_id: usize) {
        self.handles.insert(string_id, variable_id);
        trace!("{:?}", self);
    }

    pub fn get_parent_handle(&self) -> Option<usize> {
        self.parent_id
    }

    pub fn expire(&mut self) {
        self.is_expired = true;
    }
}
