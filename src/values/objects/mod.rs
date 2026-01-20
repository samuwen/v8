mod function;
mod ordinary;

use function::*;
use ordinary::*;
use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    stmt::Stmt,
    values::{JSResult, JSValue},
};

#[derive(Clone, Debug)]
pub enum JSObject {
    Ordinary(OrdinaryObject),
    Function(FunctionObject),
}

impl JSObject {
    pub fn new_ordinary_object(interpreter: &mut Interpreter) -> usize {
        let object = JSObject::Ordinary(OrdinaryObject::new());
        interpreter.object_heap.add_new_item(object)
    }

    pub fn new_function_object(interpreter: &mut Interpreter, call: Box<Stmt>) -> usize {
        todo!()
    }

    pub fn get_prototype_of(&self) -> &Option<usize> {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.get_prototype_of(),
            JSObject::Function(function_object) => function_object.get_prototype_of(),
        }
    }

    pub fn set_prototype_of(&mut self, prototype: Option<usize>) -> JSResult<bool> {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.set_prototype_of(prototype),
            JSObject::Function(function_object) => function_object.set_prototype_of(prototype),
        }
    }

    pub fn is_extensible(&self) -> bool {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.is_extensible(),
            JSObject::Function(function_object) => function_object.is_extensible(),
        }
    }

    pub fn prevent_extensible(&mut self) -> bool {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.prevent_extensible(),
            JSObject::Function(function_object) => function_object.prevent_extensible(),
        }
    }

    pub fn has_property(&self, key: &SymbolU32, interpreter: &mut Interpreter) -> JSResult<bool> {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.has_property(key, interpreter),
            JSObject::Function(function_object) => function_object.has_property(key, interpreter),
        }
    }

    pub fn get_own_property(&self, key: &SymbolU32) -> JSResult<Option<&ObjectProperty>> {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.get_own_property(key),
            JSObject::Function(function_object) => function_object.get_own_property(key),
        }
    }

    pub fn define_own_property(
        &mut self,
        key: &SymbolU32,
        value: ObjectProperty,
    ) -> JSResult<bool> {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.define_own_property(key, value),
            JSObject::Function(function_object) => function_object.define_own_property(key, value),
        }
    }

    pub fn delete(&mut self, key: &SymbolU32) -> JSResult<bool> {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.delete(key),
            JSObject::Function(function_object) => function_object.delete(key),
        }
    }

    pub fn own_property_keys(&mut self) -> JSResult<Vec<&SymbolU32>> {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.own_property_keys(),
            JSObject::Function(function_object) => function_object.own_property_keys(),
        }
    }

    pub fn get(
        &self,
        key: &SymbolU32,
        receiver: &JSValue,
        interpreter: &mut Interpreter,
    ) -> JSResult<JSValue> {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.get(key, receiver, interpreter),
            JSObject::Function(function_object) => function_object.get(key, receiver, interpreter),
        }
    }

    pub fn set(
        &mut self,
        key: &SymbolU32,
        value: &JSValue,
        receiver: &JSValue,
        interpreter: &mut Interpreter,
    ) -> JSResult<bool> {
        match self {
            JSObject::Ordinary(ordinary_object) => {
                ordinary_object.set(key, value, receiver, interpreter)
            }
            JSObject::Function(function_object) => {
                function_object.set(key, value, receiver, interpreter)
            }
        }
    }

    pub fn create_data_property(&mut self, key: &SymbolU32, value: &JSValue) -> JSResult<bool> {
        todo!()
    }

    pub fn call(&self, _value: &JSValue, _arguments: Vec<&JSValue>) -> JSResult<&JSValue> {
        todo!()
    }

    pub fn to_primitive(&self) -> JSResult<JSValue> {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.to_primitive(),
            JSObject::Function(function_object) => function_object.to_primitive(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ObjectProperty {
    Data {
        value: JSValue,
        writable: bool,
        enumerable: bool,
        configurable: bool,
    },
    Attribute {
        get: Option<JSObject>,
        set: Option<JSObject>,
        enumerable: bool,
        configurable: bool,
    },
}

impl ObjectProperty {
    pub fn new_from_value(value: JSValue) -> Self {
        Self::Data {
            value,
            writable: true,
            enumerable: true,
            configurable: true,
        }
    }

    pub fn is_configurable(&self) -> bool {
        match self {
            ObjectProperty::Data {
                value: _,
                writable: _,
                enumerable: _,
                configurable,
            } => *configurable,
            ObjectProperty::Attribute {
                get: _,
                set: _,
                enumerable: _,
                configurable,
            } => *configurable,
        }
    }
}
