use std::collections::HashMap;

use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    global::get_or_intern_string,
    values::{
        JSResult, JSValue,
        objects::{ObjectProperty, Properties},
    },
};

// https://262.ecma-international.org/15.0/index.html#sec-arraycreate
#[derive(Clone, Debug)]
pub struct Array {
    extensible: bool,
    prototype: Option<usize>,
    properties: HashMap<SymbolU32, ObjectProperty>,
}

impl Array {
    pub fn new(properties: Properties, interpreter: &mut Interpreter) -> Self {
        let map = HashMap::from_iter(
            properties
                .into_iter()
                .map(|(k, v)| (k, ObjectProperty::new_from_value(v))),
        );
        Self {
            extensible: true,
            prototype: None,
            properties: map,
        }
    }

    pub fn get_property(&self, key: &SymbolU32) -> Option<&ObjectProperty> {
        self.properties.get(key)
    }

    pub fn push(&mut self, value: JSValue) -> JSResult<JSValue> {
        let next_id = self.properties.len().to_string();
        let id = get_or_intern_string(&next_id);
        let property = ObjectProperty::new_from_value(value);
        self.properties.insert(id, property);
        let new_len = self.properties.len() as f64;
        let val = JSValue::new_number(&new_len);
        Ok(val)
    }
}
