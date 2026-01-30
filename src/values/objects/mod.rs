mod function;
mod ordinary;

use function::*;
use ordinary::*;
use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    stmt::Stmt,
    values::{JSResult, JSValue, PreferredType},
};

pub type ObjectId = usize;
pub type Properties = Vec<(SymbolU32, JSValue)>;
pub const TO_PRIMITIVE_SYM: &'static str = "@@toPrimitive";
pub const CALL: &'static str = "call";

#[derive(Clone, Debug)]
pub enum JSObject {
    Ordinary(OrdinaryObject),
    Function(FunctionObject),
}

impl JSObject {
    pub fn new_ordinary_object(properties: Properties, interpreter: &mut Interpreter) -> usize {
        let object = JSObject::Ordinary(OrdinaryObject::new(properties, interpreter));
        interpreter.object_heap.add_new_item(object)
    }

    pub fn new_function_object(interpreter: &mut Interpreter, call: Box<Stmt>) -> usize {
        todo!()
    }

    pub fn to_primitive(&self, hint: PreferredType) -> JSResult<JSValue> {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.to_primitive(hint),
            JSObject::Function(function_object) => function_object.to_primitive(hint),
        }
    }

    pub fn is_function(&self) -> bool {
        match self {
            JSObject::Ordinary(ordinary_object) => false,
            JSObject::Function(function_object) => true,
        }
    }

    pub fn value_of(&self) -> JSResult<JSValue> {
        match self {
            JSObject::Ordinary(ordinary) => self.value_of(),
            JSObject::Function(function) => todo!(),
        }
    }

    pub fn to_string(&self) -> JSResult<JSValue> {
        match self {
            JSObject::Ordinary(ordinary) => ordinary.to_string(),
            JSObject::Function(function) => todo!(),
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

    pub fn get_value(&self) -> JSResult<&JSValue> {
        match self {
            Self::Data {
                value,
                writable: _,
                enumerable: _,
                configurable: _,
            } => return Ok(value),
            _ => unimplemented!(),
        }
    }
}
