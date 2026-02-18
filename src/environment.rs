use std::collections::HashMap;

use log::trace;
use string_interner::{Symbol, symbol::SymbolU32};

use crate::{global::get_string_from_pool, variable::VariableId};

type StringId = SymbolU32;

#[derive(Debug, Clone)]
pub struct Environment {
    _is_expired: bool,
    handles: HashMap<StringId, VariableId>, // stringID: variableID (maps string names to variable ids)
}

impl Environment {
    pub fn new() -> Self {
        Self {
            _is_expired: false,
            handles: HashMap::new(),
        }
    }

    pub fn get_variable(&self, string_id: StringId) -> Option<VariableId> {
        let maybe_handle = self.handles.get(&string_id);
        if let Some(handle) = maybe_handle {
            return Some(*handle);
        }
        None
    }

    pub fn add_variable(&mut self, string_id: StringId, variable_id: VariableId) {
        self.handles.insert(string_id, variable_id);
        trace!("{:?}", self);
    }

    pub fn _expire(&mut self) {
        self._is_expired = true;
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Environment: {{")?;
        writeln!(f, "\tHandles(str_id, var_id): [")?;
        for (str_id, var_id) in self.handles.iter() {
            let uid = str_id.to_usize();
            let string = get_string_from_pool(str_id).unwrap();
            writeln!(f, "\t\t({uid}){string}: {var_id}")?;
        }
        writeln!(f, "\t]")?;
        writeln!(f, "}}")
    }
}
