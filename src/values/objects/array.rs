use std::collections::HashMap;

use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    errors::JSError,
    global::{get_or_intern_string, get_string_from_pool},
    values::{
        JSResult, JSValue, PreferredType,
        objects::{ObjectProperty, Properties, TO_PRIMITIVE_SYM},
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

    pub fn get_property_mut(&mut self, key: &SymbolU32) -> Option<&mut ObjectProperty> {
        self.properties.get_mut(key)
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

    pub fn pop(&mut self) -> JSResult<JSValue> {
        if self.properties.len() == 0 {
            return Ok(JSValue::Undefined);
        }
        let prev_id_str = (self.properties.len() - 1).to_string();
        let id = get_or_intern_string(&prev_id_str);
        let res = self.properties.remove(&id).expect("Something catastrophic"); // safe - we know there's at least 1 ID
        let value = res.get_value()?;
        Ok(value.clone())
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

    pub fn value_of(&self) -> JSResult<JSValue> {
        // Ok(JSValue::object_shallow_copy(self.id))
        todo!()
    }

    pub fn to_string(&self, interpreter: &mut Interpreter) -> JSResult<JSValue> {
        let values = self
            .properties
            .values()
            .map(|prop| {
                let val = prop.get_value()?;
                let res = val.to_string(interpreter)?;
                let string = get_string_from_pool(&res)
                    .expect("An array has a value that doesn't exist in the string pool?");
                Ok(string)
            })
            .collect::<JSResult<Vec<String>>>()?
            .join(",");
        let sym = get_or_intern_string(&format!("[{values}]"));
        Ok(JSValue::new_string(&sym))
    }
}
