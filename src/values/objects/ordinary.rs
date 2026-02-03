#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashMap;

use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    errors::JSError,
    global::get_or_intern_string,
    values::{
        JSResult, JSValue, PreferredType,
        objects::{ObjectId, ObjectProperty, Properties, TO_PRIMITIVE_SYM},
    },
};

#[derive(Clone, Debug)]
pub struct OrdinaryObject {
    id: ObjectId,
    extensible: bool,
    prototype: Option<usize>,
    properties: HashMap<SymbolU32, ObjectProperty>,
}

impl OrdinaryObject {
    pub fn new(properties: Properties, interpreter: &mut Interpreter) -> Self {
        let id = interpreter.object_heap.get_next_id();
        let map = HashMap::from_iter(
            properties
                .into_iter()
                .map(|(k, v)| (k, ObjectProperty::new_from_value(v))),
        );
        Self {
            id,
            extensible: true,
            prototype: None,
            properties: map,
        }
    }

    pub fn to_primitive(&self, hint: PreferredType) -> JSResult<JSValue> {
        let prim_sym = get_or_intern_string(TO_PRIMITIVE_SYM);
        let maybe_property = self.properties.get(&prim_sym);
        match maybe_property {
            Some(_property) => {
                todo!();
            }
            None => {
                let method_names = match hint {
                    PreferredType::Number => vec!["value_of", "to_string"],
                    PreferredType::String => vec!["to_string", "value_of"],
                };
                for method in method_names {
                    if method == "value_of" {
                        let result = self.value_of()?;
                        if !result.is_object() {
                            return Ok(result);
                        }
                    }
                    if method == "to_string" {
                        return self.to_string();
                    }
                }
                let error = JSError::new_function_type_error("unknown");
                return Err(error);
            }
        }
    }

    pub fn to_string(&self) -> JSResult<JSValue> {
        let sym = get_or_intern_string("[object Object]");
        Ok(JSValue::new_string(&sym))
    }

    pub fn value_of(&self) -> JSResult<JSValue> {
        Ok(JSValue::object_shallow_copy(self.id))
    }
}
