use std::collections::HashMap;

use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    values::{
        JSResult, JSValue,
        objects::{ObjectProperty, function::FunctionObject},
    },
};

#[derive(Clone, Debug)]
pub struct OrdinaryObject {
    extensible: bool,
    prototype: Option<usize>,
    properties: HashMap<SymbolU32, ObjectProperty>,
}

impl OrdinaryObject {
    pub fn new() -> Self {
        let map = HashMap::new();
        Self {
            extensible: true,
            prototype: None,
            properties: map,
        }
    }

    pub fn get_prototype_of(&self) -> &Option<usize> {
        &self.prototype
    }

    pub fn set_prototype_of(&mut self, prototype: Option<usize>) -> JSResult<bool> {
        self.prototype = prototype;
        Ok(true)
    }

    pub fn is_extensible(&self) -> bool {
        self.extensible
    }

    pub fn prevent_extensible(&mut self) -> bool {
        if self.extensible {
            self.extensible = false;
            return true;
        }
        false
    }

    pub fn get_own_property(&self, key: &SymbolU32) -> JSResult<Option<&ObjectProperty>> {
        Ok(self.properties.get(key))
    }

    pub fn define_own_property(
        &mut self,
        key: &SymbolU32,
        value: ObjectProperty,
    ) -> JSResult<bool> {
        if self.is_extensible() {
            return Ok(self.properties.insert(*key, value).is_some());
        }
        Ok(false)
    }

    pub fn has_property(&self, key: &SymbolU32, interpreter: &mut Interpreter) -> JSResult<bool> {
        let own_prop = self.properties.contains_key(key);
        if own_prop {
            return Ok(true);
        }
        if let Some(proto) = interpreter
            .object_heap
            .get_item_from_option(&self.prototype)
        {
            return proto.has_property(key, interpreter);
        }
        Ok(false)
    }

    pub fn get(
        &self,
        key: &SymbolU32,
        receiver: &JSValue,
        interpreter: &mut Interpreter,
    ) -> JSResult<JSValue> {
        let own_property = self.get_own_property(key)?;
        match own_property {
            Some(desc) => match desc {
                ObjectProperty::Data {
                    value,
                    writable: _,
                    enumerable: _,
                    configurable: _,
                } => {
                    return Ok(value.clone());
                }
                ObjectProperty::Attribute {
                    get,
                    set: _,
                    enumerable: _,
                    configurable: _,
                } => {
                    if let Some(get) = get {
                        todo!()
                        // return get.call(receiver, vec![]);
                    }
                    return Ok(JSValue::Undefined);
                }
            },
            None => {
                let parent = self.get_prototype_of();
                if let Some(proto) = interpreter.object_heap.get_item_from_option(&parent) {
                    return proto.get(key, receiver, interpreter);
                }
                Ok(JSValue::Undefined)
            }
        }
    }

    pub fn set(
        &mut self,
        key: &SymbolU32,
        value: &JSValue,
        receiver: &JSValue,
        interpreter: &mut Interpreter,
    ) -> JSResult<bool> {
        let own_desc = self.get_own_property(key)?;
        let own_desc = if let None = own_desc {
            let parent = self.get_prototype_of();
            if let Some(mut proto) = interpreter.object_heap.get_item_from_option(&parent) {
                return proto.set(key, value, receiver, interpreter);
            } else {
                &ObjectProperty::Data {
                    value: JSValue::Undefined,
                    writable: true,
                    enumerable: true,
                    configurable: true,
                }
            }
        } else {
            own_desc.unwrap()
        };

        match own_desc {
            ObjectProperty::Data {
                value,
                writable,
                enumerable: _,
                configurable: _,
            } => {
                if !*writable {
                    return Ok(false);
                }
                match receiver {
                    JSValue::Object {
                        object_id: receiver_id,
                    } => {
                        let mut receiver = interpreter.object_heap.get_item_from_id(*receiver_id);
                        let existing_descriptor = receiver.get_own_property(key)?;
                        match existing_descriptor {
                            Some(descriptor) => {
                                if let ObjectProperty::Data {
                                    value,
                                    writable,
                                    enumerable: _,
                                    configurable: _,
                                } = descriptor
                                {
                                    if !writable {
                                        return Ok(false);
                                    }
                                    let value_desc = ObjectProperty::new_from_value(value.clone());
                                    return receiver.define_own_property(key, value_desc);
                                }
                                return Ok(false);
                            }
                            None => {
                                return receiver.create_data_property(key, value);
                            }
                        }
                    }
                    _ => return Ok(false),
                }
            }
            ObjectProperty::Attribute {
                get: _,
                set,
                enumerable: _,
                configurable: _,
            } => {
                if let Some(setter) = set {
                    setter.call(receiver, vec![value]);
                    return Ok(true);
                }
                return Ok(false);
            }
        }
    }

    pub fn delete(&mut self, key: &SymbolU32) -> JSResult<bool> {
        let desc = self.get_own_property(key)?;
        if let Some(d) = desc {
            if d.is_configurable() {
                self.properties.remove(key);
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn own_property_keys(&self) -> JSResult<Vec<&SymbolU32>> {
        let k = self.properties.keys();
        let keys = k.collect();

        Ok(keys)
    }

    pub fn to_primitive(&self) -> JSResult<JSValue> {
        todo!()
    }
}
