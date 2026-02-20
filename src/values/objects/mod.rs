#![allow(dead_code)]
#![allow(unused_variables)]

mod array;
mod function;
mod ordinary;

use core::f64;

use function::*;
use log::debug;
use ordinary::*;
use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    constants::{
        CONSOLE_NAME, ERROR_NAME, GLOBAL_THIS_NAME, INFINITY_NAME, IS_FINITE_NAME, LOG_NAME,
        NAN_NAME, UNDEFINED_NAME,
    },
    errors::JSError,
    expr::Expr,
    global::{get_or_intern_string, get_string_from_pool},
    stmt::Stmt,
    values::{JSResult, JSValue, ObjectKind, PreferredType, objects::array::Array},
};

pub type ObjectId = usize;
pub type Property = (SymbolU32, JSValue);
pub type Properties = Vec<Property>;
pub const TO_PRIMITIVE_SYM: &'static str = "@@toPrimitive";

#[derive(Clone, Debug)]
pub enum JSObject {
    Ordinary(OrdinaryObject),
    Function(FunctionObject),
    Array(Array),
}

impl JSObject {
    pub fn new_ordinary_object(
        properties: Properties,
        extensible: bool,
        proto: Option<usize>,
        interpreter: &mut Interpreter,
    ) -> usize {
        let ordinary = OrdinaryObject::new(properties, extensible, proto);
        let object = JSObject::Ordinary(ordinary);
        interpreter.add_object(object)
    }

    pub fn new_function_object(
        call: Box<Stmt>,
        params: Vec<SymbolU32>,
        environment_id: usize,
        interpreter: &mut Interpreter,
    ) -> usize {
        let proto_id = interpreter.function_proto_id;
        let object =
            JSObject::Function(FunctionObject::new(call, environment_id, proto_id, params));
        interpreter.add_object(object)
    }

    pub fn new_array_object(properties: Properties, interpreter: &mut Interpreter) -> usize {
        let ordinary = Array::new(properties, interpreter);
        let object = JSObject::Array(ordinary);
        interpreter.add_object(object)
    }

    pub fn create_object_proto() -> Self {
        let ordinary = OrdinaryObject::new(vec![], true, None);
        JSObject::Ordinary(ordinary)
    }

    pub fn create_function_proto(env_id: usize, proto_id: usize) -> Self {
        let function = FunctionObject::new_proto(env_id, proto_id);
        JSObject::Function(function)
    }

    pub fn create_global_object(interpreter: &mut Interpreter) {
        let proto_id = interpreter.get_object_proto_id();
        let mut global_object = OrdinaryObject::new(vec![], true, Some(proto_id));

        let inf_str_id = get_or_intern_string(INFINITY_NAME);
        let infinity = JSValue::new_number(&f64::INFINITY);
        global_object.add_property_from_value(inf_str_id, infinity);

        let nan_str_id = get_or_intern_string(NAN_NAME);
        let nan = JSValue::new_number(&f64::NAN);
        global_object.add_property_from_value(nan_str_id, nan);

        let undef_str_id = get_or_intern_string(UNDEFINED_NAME);
        let undefined = JSValue::new_undefined();
        global_object.add_property_from_value(undef_str_id, undefined);

        let (is_finite_id, is_finite) = JSObject::new_built_in_fn(
            IS_FINITE_NAME,
            FunctionObject::create_is_finite,
            interpreter,
        );
        global_object.add_property(is_finite_id, ObjectPropertyBuilder::new(is_finite).build());

        let (console_id, console_obj) = JSObject::new_built_in_obj(
            CONSOLE_NAME,
            vec![
                JSObject::new_built_in_fn(LOG_NAME, FunctionObject::create_log, interpreter),
                JSObject::new_built_in_fn(ERROR_NAME, FunctionObject::create_error, interpreter),
            ],
            interpreter,
        );
        global_object.add_property(console_id, ObjectPropertyBuilder::new(console_obj).build());

        let global_object = JSObject::Ordinary(global_object);
        let obj_id = interpreter.add_object(global_object);
        let value = JSValue::Object {
            object_id: obj_id,
            kind: ObjectKind::Object,
        };
        let global_this_id = get_or_intern_string(GLOBAL_THIS_NAME);
        interpreter.new_variable(global_this_id, false, value);
    }

    pub fn to_primitive(
        &self,
        hint: PreferredType,
        interpreter: &mut Interpreter,
    ) -> JSResult<JSValue> {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.to_primitive(hint, interpreter),
            JSObject::Function(function_object) => function_object.to_primitive(hint),
            JSObject::Array(array) => array.to_primitive(hint, interpreter),
        }
    }

    pub fn is_function(&self) -> bool {
        match self {
            JSObject::Function(_) => true,
            _ => false,
        }
    }

    pub fn value_of(&self) -> JSResult<JSValue> {
        match self {
            JSObject::Ordinary(ordinary) => ordinary.value_of(),
            JSObject::Function(function) => todo!(),
            JSObject::Array(array) => todo!(),
        }
    }

    pub fn call(
        &self,
        args: &Vec<Expr>,
        name: Option<&SymbolU32>,
        interpreter: &mut Interpreter,
    ) -> JSResult<JSValue> {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.call(name.unwrap()),
            JSObject::Function(object) => object.call(args, interpreter),
            JSObject::Array(array) => todo!(),
        }
    }

    pub fn get_property(&self, key: &SymbolU32) -> Option<&ObjectProperty> {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.get_property(key),
            JSObject::Function(function_object) => todo!(),
            JSObject::Array(array) => array.get_property(key),
        }
    }

    pub fn get_property_mut(&mut self, key: &SymbolU32) -> Option<&mut ObjectProperty> {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.get_property_mut(key),
            JSObject::Function(function_object) => todo!(),
            JSObject::Array(array) => array.get_property_mut(key),
        }
    }

    pub fn add_property(&mut self, key: SymbolU32, value: JSValue) {
        let prop = ObjectProperty::new_from_value(value);
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.add_property(key, prop),
            JSObject::Function(function_object) => todo!(),
            JSObject::Array(array) => todo!(),
        }
    }

    pub fn debug(&self, interpreter: &mut Interpreter) -> String {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.debug(interpreter),
            JSObject::Function(function_object) => function_object.debug(interpreter),
            JSObject::Array(array) => todo!(),
        }
    }

    pub fn new_built_in_fn<F>(name: &str, f: F, interpreter: &mut Interpreter) -> Property
    where
        F: Fn(&mut Interpreter) -> FunctionObject,
    {
        let str_id = get_or_intern_string(name);
        debug!("{name} has id {str_id:?}");
        let fn_object = f(interpreter);
        let js_object = JSObject::Function(fn_object);
        let object_id = interpreter.add_object(js_object);
        let js_value = JSValue::Object {
            object_id,
            kind: ObjectKind::Function,
        };
        (str_id, js_value)
    }

    fn new_built_in_obj(
        name: &str,
        properties: Properties,
        interpreter: &mut Interpreter,
    ) -> Property {
        let str_id = get_or_intern_string(name);
        let object_id = JSObject::new_ordinary_object(properties, true, None, interpreter);
        let js_value = JSValue::Object {
            object_id,
            kind: ObjectKind::Object,
        };
        (str_id, js_value)
    }
}

pub fn get_object_property<'a>(
    interpreter: &'a mut Interpreter,
    object_value: &JSValue,
    key: SymbolU32,
) -> JSResult<&'a ObjectProperty> {
    if let JSValue::Object { object_id, kind: _ } = object_value {
        let object = interpreter.get_object(*object_id)?;
        let key_string = get_string_from_pool(&key).unwrap();

        let property = object.get_property(&key);
        if let Some(prop) = property {
            return Ok(prop);
        }
    }

    Err(JSError::new("Could not get object property"))
}

pub fn get_object_property_mut<'a>(
    interpreter: &'a mut Interpreter,
    object_value: &JSValue,
    key: SymbolU32,
) -> JSResult<&'a mut ObjectProperty> {
    if let JSValue::Object { object_id, kind: _ } = object_value {
        let object = interpreter.get_object_mut(*object_id)?;
        let property_opt = object.get_property_mut(&key);
        if let Some(property) = property_opt {
            return Ok(property);
        }
    }

    Err(JSError::new("Could not get object property"))
}

struct ObjectPropertyBuilder {
    value: JSValue,
    writable: Option<bool>,
    enumerable: Option<bool>,
    configurable: Option<bool>,
}

impl ObjectPropertyBuilder {
    fn new(value: JSValue) -> Self {
        Self {
            value,
            writable: None,
            enumerable: None,
            configurable: None,
        }
    }

    fn writable(mut self, w: bool) -> Self {
        self.writable = Some(w);
        self
    }

    fn enumerable(mut self, w: bool) -> Self {
        self.enumerable = Some(w);
        self
    }

    fn configurable(mut self, w: bool) -> Self {
        self.configurable = Some(w);
        self
    }

    fn build(self) -> ObjectProperty {
        ObjectProperty::Data {
            value: self.value,
            writable: self.writable.unwrap_or_default(),
            enumerable: self.enumerable.unwrap_or_default(),
            configurable: self.configurable.unwrap_or_default(),
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

    pub fn set_value(&mut self, value: JSValue) {
        let old_value = match self {
            Self::Data {
                value,
                writable,
                enumerable,
                configurable,
            } => value,
            _ => unimplemented!(),
        };
        *old_value = value;
    }
}

impl std::fmt::Display for JSObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JSObject::Ordinary(ordinary_object) => write!(f, "{ordinary_object}"),
            JSObject::Function(function_object) => write!(f, "{function_object}"),
            JSObject::Array(array) => todo!(),
        }
    }
}
