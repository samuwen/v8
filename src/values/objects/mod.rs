#![allow(dead_code)]
#![allow(unused_variables)]

mod array;
mod function;
mod ordinary;

use function::*;
use ordinary::*;
use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    expr::Expr,
    global::get_or_intern_string,
    stmt::Stmt,
    values::{JSResult, JSValue, PreferredType, objects::array::Array},
};

pub type ObjectId = usize;
pub type Properties = Vec<(SymbolU32, JSValue)>;
pub const TO_PRIMITIVE_SYM: &'static str = "@@toPrimitive";
pub const CALL: &'static str = "call";

#[derive(Clone, Debug)]
pub enum JSObject {
    Ordinary(OrdinaryObject),
    Function(FunctionObject),
    Array(Array),
}

impl JSObject {
    pub fn new_ordinary_object(properties: Properties, interpreter: &mut Interpreter) -> usize {
        let ordinary = OrdinaryObject::new(properties, interpreter);
        let object = JSObject::Ordinary(ordinary);
        interpreter.add_object(object)
    }

    pub fn new_function_object(
        call: Box<Stmt>,
        params: Vec<SymbolU32>,
        environment_id: usize,
        interpreter: &mut Interpreter,
    ) -> usize {
        let object = JSObject::Function(FunctionObject::new(call, environment_id, params));
        interpreter.add_object(object)
    }

    pub fn new_array_object(properties: Properties, interpreter: &mut Interpreter) -> usize {
        let ordinary = Array::new(properties, interpreter);
        let object = JSObject::Array(ordinary);
        interpreter.add_object(object)
    }

    /// Setup the initial global object. Initially just for console
    pub fn create_global_this(interpreter: &mut Interpreter) {
        let print_expr = Expr::new_print_expr(crate::expr::LogKind::Log);
        let call_stmt = Stmt::new_expression(print_expr);
        let scope_id = interpreter.enter_scope(None);
        let data_id = get_or_intern_string("data");
        let parameters = vec![data_id];
        for param in &parameters {
            interpreter.new_variable(*param, true, JSValue::Undefined);
        }
        interpreter.leave_scope();
        let log_fn_id =
            JSObject::new_function_object(Box::new(call_stmt), parameters, scope_id, interpreter);
        let log_value = JSValue::Object {
            object_id: log_fn_id,
        };
        let console_id = get_or_intern_string("console");
        let log_id = get_or_intern_string("log");
        let console_object = JSObject::new_ordinary_object(vec![(log_id, log_value)], interpreter);
        let console_value = JSValue::Object {
            object_id: console_object,
        };
        interpreter.new_variable(console_id, false, console_value);

        // let _global_this = JSObject::new_ordinary_object(vec![(console_id, console_value)], interpreter);
    }

    pub fn to_primitive(&self, hint: PreferredType) -> JSResult<JSValue> {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.to_primitive(hint),
            JSObject::Function(function_object) => function_object.to_primitive(hint),
            JSObject::Array(array) => todo!(),
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

    pub fn to_string(&self) -> JSResult<JSValue> {
        match self {
            JSObject::Ordinary(ordinary) => ordinary.to_string(),
            JSObject::Function(function) => todo!(),
            JSObject::Array(array) => todo!(),
        }
    }

    pub fn call(
        &self,
        args: Vec<JSValue>,
        name: &SymbolU32,
        interpreter: &mut Interpreter,
    ) -> JSResult<JSValue> {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.call(name),
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

    pub fn debug(&self, interpreter: &mut Interpreter) -> String {
        match self {
            JSObject::Ordinary(ordinary_object) => ordinary_object.debug(interpreter),
            JSObject::Function(function_object) => function_object.debug(interpreter),
            JSObject::Array(array) => todo!(),
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
