#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashMap;

use log::trace;
use string_interner::symbol::SymbolU32;

use crate::{Interpreter, global::get_string_from_pool};

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

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Environment: {{")?;
        let parent_id = match self.parent_id {
            Some(id) => id.to_string(),
            None => "None".to_string(),
        };
        writeln!(f, "\tParentID: {parent_id}")?;
        writeln!(f, "\tHandles: [")?;
        for (str_id, var_id) in self.handles.iter() {
            let string = get_string_from_pool(str_id).unwrap();
            writeln!(f, "\t\t{string}: {var_id}")?;
        }
        writeln!(f, "\t]")?;
        writeln!(f, "}}")
    }
}
