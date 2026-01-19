use std::collections::HashMap;

use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    stmt::Stmt,
    values::{
        JSResult, JSValue,
        objects::{JSObject, ObjectProperty},
    },
};

#[derive(Clone, Debug)]
pub struct FunctionObject {
    name: SymbolU32,
    prototype: Option<usize>,
    property_map: HashMap<SymbolU32, ObjectProperty>,
    call: Box<Stmt>, // create the statement wrapper around it before passing it thru
    environment: SymbolU32, // whenever i figure out lexical scopes
    formal_parameters: Vec<SymbolU32>,
}

impl FunctionObject {
    pub fn new(
        ident: SymbolU32,
        prototype: Option<usize>,
        call: Box<Stmt>,
        environment: SymbolU32,
        parameters: Vec<SymbolU32>,
    ) -> Self {
        let map = HashMap::new();
        Self {
            name: ident,
            prototype,
            property_map: map,
            call,
            environment,
            formal_parameters: parameters,
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
        false
    }

    pub fn prevent_extensible(&mut self) -> bool {
        false
    }

    pub fn get_own_property(&self, key: &SymbolU32) -> JSResult<Option<&ObjectProperty>> {
        Ok(self.property_map.get(key))
    }

    pub fn define_own_property(
        &mut self,
        key: &SymbolU32,
        value: ObjectProperty,
    ) -> JSResult<bool> {
        if self.is_extensible() {
            return Ok(self.property_map.insert(*key, value).is_some());
        }
        Ok(false)
    }

    pub fn has_property(&self, key: &SymbolU32, interpreter: &mut Interpreter) -> JSResult<bool> {
        let own_prop = self.property_map.contains_key(key);
        if own_prop {
            return Ok(true);
        }
        if let Some(proto_id) = &self.prototype {
            let proto = interpreter.heap.get_object_from_id(*proto_id);
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
                if let Some(proto) = interpreter.heap.get_object_from_option(&parent) {
                    return proto.get(key, receiver, interpreter);
                }
                Ok(JSValue::Undefined)
            }
        }
    }

    pub fn set(&mut self, key: &SymbolU32, value: &JSValue, receiver: &JSValue) -> JSResult<bool> {
        todo!()
        // let own_desc = self.get_own_property(key)?;
        // let own_desc = if let None = own_desc {
        //     let parent = self.get_prototype_of();
        //     match parent {
        //         Some(mut proto) => {
        //             return proto.set(key, value, receiver);
        //         }
        //         None => &ObjectProperty::Data {
        //             value: JSValue::Undefined,
        //             writable: true,
        //             enumerable: true,
        //             configurable: true,
        //         },
        //     }
        // } else {
        //     own_desc.unwrap()
        // };

        // match own_desc {
        //     ObjectProperty::Data {
        //         value,
        //         writable,
        //         enumerable,
        //         configurable,
        //     } => {
        //         if !*writable {
        //             return Ok(false);
        //         }
        //         match receiver {
        //             JSValue::Object(receiver) => {
        //                 let existing_descriptor = receiver.get_own_property(key)?;
        //                 match existing_descriptor {
        //                     Some(descriptor) => {
        //                         if let ObjectProperty::Data {
        //                             value,
        //                             writable,
        //                             enumerable,
        //                             configurable,
        //                         } = descriptor
        //                         {
        //                             if !writable {
        //                                 return Ok(false);
        //                             }
        //                             let value_desc = ObjectProperty::new_from_value(value.clone());
        //                             return receiver.define_own_property(key, value_desc);
        //                         }
        //                         return Ok(false);
        //                     }
        //                     None => {
        //                         return receiver.create_data_property(key, value);
        //                     }
        //                 }
        //             }
        //             _ => return Ok(false),
        //         }
        //     }
        //     ObjectProperty::Attribute {
        //         get,
        //         set,
        //         enumerable,
        //         configurable,
        //     } => {
        //         if let Some(setter) = set {
        //             setter.call(receiver, vec![value]);
        //             return Ok(true);
        //         }
        //         return Ok(false);
        //     }
        // }
    }

    pub fn delete(&mut self, key: &SymbolU32) -> JSResult<bool> {
        let desc = self.get_own_property(key)?;
        if let Some(d) = desc {
            if d.is_configurable() {
                self.property_map.remove(key);
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn own_property_keys(&self) -> JSResult<Vec<&SymbolU32>> {
        let k = self.property_map.keys();
        let keys = k.collect();

        Ok(keys)
    }

    pub fn call(&self, _this_argument: &JSValue, _arguments: Vec<&JSValue>) {
        todo!()
    }
}
