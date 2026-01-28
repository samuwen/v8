use std::collections::HashMap;

use string_interner::symbol::SymbolU32;

use crate::Interpreter;

type EnvironmentId = usize;
type StringId = SymbolU32;

#[derive(Debug, Clone)]
pub struct Environment {
    parent_id: Option<EnvironmentId>,
    handles: HashMap<StringId, usize>, // stringID: variableID (maps string names to variable ids)
}

impl Environment {
    pub fn new(parent_id: Option<EnvironmentId>) -> Self {
        Self {
            parent_id,
            handles: HashMap::new(),
        }
    }

    pub fn has_variable(&self, string_id: StringId, interpreter: &Interpreter) -> bool {
        if self.handles.contains_key(&string_id) {
            return true;
        }
        if let Some(parent) = self.parent_id {
            let parent_env = interpreter.environment_heap.get_item_from_id(parent);
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
            let parent_env = interpreter.environment_heap.get_item_from_id(parent);
            return parent_env.get_variable(string_id, interpreter);
        }
        None
    }

    pub fn add_variable(&mut self, string_id: StringId, variable_id: usize) {
        self.handles.insert(string_id, variable_id);
    }
}
