use std::collections::HashMap;

use string_interner::{Symbol, symbol::SymbolU32};

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
    pub fn new(properties: Properties, extensible: bool, proto: Option<usize>) -> Self {
        let map = HashMap::from_iter(
            properties
                .into_iter()
                .map(|(k, v)| (k, ObjectProperty::new_from_value(v))),
        );
        Self {
            extensible,
            prototype: proto,
            properties: map,
        }
    }

    pub fn new_from_full_props(
        properties: Vec<(SymbolU32, ObjectProperty)>,
        extensible: bool,
        proto: Option<usize>,
        interpreter: &mut Interpreter,
    ) -> Self {
        Self {
            extensible,
            prototype: proto,
            properties: HashMap::from_iter(properties),
        }
    }

    pub fn to_primitive(
        &self,
        hint: PreferredType,
        interpreter: &mut Interpreter,
    ) -> JSResult<JSValue> {
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
                        return self.to_string(interpreter);
                    }
                }
                let error = JSError::new_function_type_error("unknown");
                return Err(error);
            }
        }
    }

    pub fn to_string(&self, interpreter: &mut Interpreter) -> JSResult<JSValue> {
        let values = self
            .properties
            .iter()
            .map(|(key, prop)| {
                let key = get_string_from_pool(key).unwrap();
                let val = prop.get_value()?;
                let res = val.to_string(interpreter)?;
                let string = get_string_from_pool(&res)
                    .expect("An object has a value that doesn't exist in the string pool?");
                Ok(format!("{key}: {string}"))
            })
            .collect::<JSResult<Vec<String>>>()?
            .join(", ");
        let values = if values.len() > 70 {
            let mut new_values = String::new();
            let mut found = false;
            for c in values.chars() {
                if found {
                    new_values.push('\n');
                    found = false;
                }
                if c == ',' || c == '{' || c == '}' {
                    found = true;
                }
                new_values.push(c);
            }
            new_values
        } else {
            values
        };
        let sym = get_or_intern_string(&format!("{{{values}}}"));
        Ok(JSValue::new_string(&sym))
    }

    pub fn value_of(&self) -> JSResult<JSValue> {
        // Ok(JSValue::object_shallow_copy(self.id))
        todo!()
    }

    pub fn get_property(&self, key: &SymbolU32) -> Option<&ObjectProperty> {
        self.properties.get(key)
    }

    pub fn get_property_mut(&mut self, key: &SymbolU32) -> Option<&mut ObjectProperty> {
        self.properties.get_mut(key)
    }

    pub fn add_property(&mut self, key: SymbolU32, value: ObjectProperty) {
        self.properties.insert(key, value);
    }

    pub fn add_property_from_value(&mut self, key: SymbolU32, value: JSValue) {
        self.properties
            .insert(key, ObjectProperty::new_from_value(value));
    }

    pub fn call(&self, name: &SymbolU32) -> JSResult<JSValue> {
        let s = get_string_from_pool(name).unwrap_or("anonymous".to_string());
        let error = JSError::new(&format!("Uncaught TypeError: {s} is not a function"));
        Err(error)
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

impl std::fmt::Display for OrdinaryObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Object: {{")?;
        for (key, value) in &self.properties {
            let string = get_string_from_pool(&key).unwrap();
            let key_index = key.to_usize();
            writeln!(f, "\t ({key_index}){string}: {:?}", value.get_value())?;
        }
        writeln!(f, "}}")
    }
}
