#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashMap;

use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter, debug_value,
    errors::JSError,
    global::{get_or_intern_string, get_string_from_pool},
    values::{
        JSResult, JSValue, PreferredType,
        objects::{ObjectProperty, Properties, TO_PRIMITIVE_SYM},
    },
};

#[derive(Clone, Debug)]
pub struct OrdinaryObject {
    extensible: bool,
    prototype: Option<usize>,
    properties: HashMap<SymbolU32, ObjectProperty>,
}

impl OrdinaryObject {
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
        // Ok(JSValue::object_shallow_copy(self.id))
        todo!()
    }

    pub fn get_property(&self, key: &SymbolU32) -> Option<&ObjectProperty> {
        self.properties.get(key)
    }

    pub fn debug(&self, interpreter: &mut Interpreter) -> String {
        let mut out = String::new();
        out.push_str("{");
        let mut props = vec![];
        for (string_id, object_property) in &self.properties {
            let string = get_string_from_pool(&string_id).unwrap_or("UNKNOWN".to_string());
            let value = object_property.get_value().unwrap().clone();
            let prop = format!(" {string}: {}", debug_value(interpreter, &value));
            props.push(prop);
        }
        let prop_string = props.join(",");
        out.push_str(&prop_string);
        out.push_str(" }");
        out
    }
}
