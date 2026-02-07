#![allow(dead_code)]
#![allow(unused_variables)]

use std::{
    mem::discriminant,
    sync::{Mutex, OnceLock},
};

use log::debug;
use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    errors::JSError,
    expr::Expr,
    global::{get_or_intern_string, get_string_from_pool},
    stmt::Stmt,
    token::{Kind, Token},
    utils::get_function_params,
    values::{
        JSResult, PreferredType, add, divide, equal, less_than, multiply,
        objects::{JSObject, ObjectId, Properties},
        remainder, subtract,
    },
};

static SYMBOL_COUNTER: OnceLock<Mutex<usize>> = OnceLock::new();

fn get_symbol_counter() -> &'static Mutex<usize> {
    SYMBOL_COUNTER.get_or_init(|| Mutex::new(0))
}

fn get_new_symbol_id() -> usize {
    let mut counter = get_symbol_counter().lock().unwrap();
    let value = counter.clone();
    *counter += 1;
    value
}

#[derive(Clone, Debug)]
pub enum ArrowFunctionReturn {
    Expr(Box<Expr>),
    Stmt(Box<Stmt>),
}

#[derive(Clone, Debug)]
pub enum JSValue {
    Null,
    Undefined,
    Boolean { data: bool },
    String { data: SymbolU32 },
    Symbol { id: usize, description: SymbolU32 },
    Number { data: f64 },
    BigInt,
    Object { object_id: usize },
}

impl JSValue {
    pub fn to_primitive(
        &self,
        preferred_type: Option<PreferredType>,
        interpreter: &mut Interpreter,
    ) -> JSResult<&JSValue> {
        match self {
            JSValue::Object { object_id } => {
                todo!()
            }
            _ => Ok(self),
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            JSValue::Boolean { data } => *data,
            JSValue::Undefined | JSValue::Null => false,
            JSValue::Number { data } => !(*data == 0.0 || *data == -0.0 || f64::is_nan(*data)),
            JSValue::String { data } => {
                let string_opt = get_string_from_pool(data);
                if let Some(string) = string_opt {
                    return !(&string == "");
                }
                true
            }
            _ => true,
        }
    }

    pub fn to_numeric(&self, interpreter: &mut Interpreter) -> JSResult<f64> {
        let prim_value = self.to_primitive(Some(PreferredType::Number), interpreter)?;
        match prim_value {
            JSValue::BigInt => todo!(),
            _ => self.to_number(interpreter),
        }
    }

    pub fn to_number(&self, interpreter: &mut Interpreter) -> JSResult<f64> {
        let res = match self {
            JSValue::Null => 0.0,
            JSValue::Undefined => f64::NAN,
            JSValue::Boolean { data } => match data {
                true => 1.0,
                false => 0.0,
            },
            JSValue::String { data } => JSValue::string_to_number(data)?,
            JSValue::Symbol {
                id: _,
                description: _,
            }
            | JSValue::BigInt => {
                return Err(JSError::new_function_type_error("unknown"));
            }
            JSValue::Number { data } => *data,
            JSValue::Object { object_id } => {
                let object = interpreter.get_object_mut(*object_id)?;
                let prim_value = object.to_primitive(PreferredType::Number)?;
                prim_value.to_number(interpreter)?
            }
        };
        Ok(res)
    }

    pub fn string_to_number(value: &SymbolU32) -> JSResult<f64> {
        todo!();
    }

    // might wanna move these elsewhere
    pub fn string_to_code_points(value: &SymbolU32) -> JSResult<Vec<u32>> {
        todo!()
    }

    pub fn to_string(&self, interpreter: &mut Interpreter) -> JSResult<SymbolU32> {
        Ok(match self {
            JSValue::Null => get_or_intern_string("null"),
            JSValue::Undefined => get_or_intern_string("undefined"),
            JSValue::Boolean { data } => get_or_intern_string(&data.to_string()),
            JSValue::String { data } => *data,
            JSValue::Symbol { id: _, description } => *description,
            JSValue::Number { data } => get_or_intern_string(&data.to_string()),
            JSValue::BigInt => todo!(),
            JSValue::Object { object_id } => {
                let object = interpreter.get_object(*object_id)?;
                let prim_value = object.to_primitive(PreferredType::String)?;
                prim_value.to_string(interpreter)?
            }
        })
    }

    pub fn is_undefined(&self) -> bool {
        discriminant(self) == discriminant(&JSValue::Undefined)
    }

    pub fn is_object(&self) -> bool {
        discriminant(self) == discriminant(&JSValue::Object { object_id: 0 })
    }

    pub fn is_string(&self) -> bool {
        match self {
            Self::String { data: _ } => true,
            _ => false,
        }
    }

    pub fn is_symbol(&self) -> bool {
        match self {
            Self::Symbol {
                id: _,
                description: _,
            } => true,
            _ => false,
        }
    }

    pub fn new_number(v: &f64) -> Self {
        Self::Number { data: *v }
    }

    pub fn new_boolean(v: &bool) -> Self {
        Self::Boolean { data: *v }
    }

    pub fn new_undefined() -> Self {
        Self::Undefined
    }

    pub fn new_null() -> Self {
        Self::Null
    }

    pub fn new_object(properties: Properties, interpreter: &mut Interpreter) -> Self {
        let object_id = JSObject::new_ordinary_object(properties, interpreter);
        Self::Object { object_id }
    }

    pub fn object_shallow_copy(id: ObjectId) -> Self {
        Self::Object { object_id: id }
    }

    pub fn get_object_id(&self) -> JSResult<usize> {
        if let JSValue::Object { object_id } = self {
            return Ok(*object_id);
        }
        Err(JSError::new("Object not found"))
    }

    pub fn get_object<'a>(&'a self, interpreter: &'a Interpreter) -> JSResult<&'a JSObject> {
        if let JSValue::Object { object_id } = self {
            return interpreter.get_object(*object_id);
        }
        Err(JSError::new("Expected object"))
    }

    pub fn new_string(s: &SymbolU32) -> Self {
        Self::String { data: *s }
    }

    pub fn new_arrow_function(
        args: Vec<Expr>,
        body: ArrowFunctionReturn,
        interpreter: &mut Interpreter,
    ) -> Self {
        match body {
            ArrowFunctionReturn::Expr(expr) => {
                let stmt = Box::new(Stmt::new_return(Some(*expr))); // just make it into a return statement
                todo!()
            }
            ArrowFunctionReturn::Stmt(stmt) => {
                todo!()
            }
        }
    }

    pub fn new_array(args: Vec<Expr>) -> Self {
        todo!()
    }

    pub fn new_function(
        ident: Option<Expr>,
        args: Vec<Expr>,
        body: Stmt,
        interpreter: &mut Interpreter,
    ) -> JSResult<Self> {
        let identifier = match ident {
            Some(i) => i.evaluate(interpreter)?,
            None => {
                let sym_id = get_new_symbol_id();
                let description = format!("unknown-function-{sym_id}");
                let desc = get_or_intern_string(&description);
                JSValue::Symbol {
                    id: get_new_symbol_id(),
                    description: desc,
                }
            }
        };
        let ident_id = identifier.to_string(interpreter)?;
        let scope_id = interpreter.enter_scope(None);
        let parameters = get_function_params(&args, interpreter)?;
        let object_id =
            JSObject::new_function_object(Box::new(body), parameters, scope_id, interpreter);
        let value = JSValue::Object { object_id };
        interpreter.new_variable(ident_id, false, value);

        Ok(JSValue::Undefined)
    }

    pub fn to_int_32(&self, interpreter: &mut Interpreter) -> JSResult<i32> {
        let number = self.to_number(interpreter)?;
        if number.is_infinite() || number == 0.0 || number == -0.0 {
            return Ok(0);
        }
        let int = number.trunc() as i32;
        let rhs_mod = 2i32.pow(32);
        let int32bit = int % rhs_mod;
        if int32bit >= 2i32.pow(31) {
            return Ok(int32bit - rhs_mod);
        }

        Ok(int32bit)
    }

    pub fn to_uint_32(&self, interpreter: &mut Interpreter) -> JSResult<u32> {
        let number = self.to_number(interpreter)?;
        if number.is_infinite() || number == 0.0 || number == -0.0 {
            return Ok(0);
        }
        let int = number.trunc() as u32;
        let rhs_mod = 2u32.pow(32);
        let int32bit = int % rhs_mod;
        Ok(int32bit)
    }

    pub fn apply_string_or_numeric_binary_operator(
        &self,
        op: &Token,
        right: &JSValue,
        interpreter: &mut Interpreter,
    ) -> JSResult<JSValue> {
        let left_prim = self.to_primitive(None, interpreter)?;
        let right_prim = right.to_primitive(None, interpreter)?;
        if left_prim.is_string() || right_prim.is_string() {
            let left_str_sym = left_prim.to_string(interpreter)?;
            let right_str_sym = right_prim.to_string(interpreter)?;
            let left_str = get_string_from_pool(&left_str_sym).unwrap(); // panic should be fine here, programmer error not JS error
            let right_str = get_string_from_pool(&right_str_sym).unwrap();
            let concatenated = format!("{left_str}{right_str}");
            let id = get_or_intern_string(&concatenated);
            return Ok(JSValue::new_string(&id));
        }
        // must be numbers at this point
        let l_num = left_prim.to_numeric(interpreter)?;
        let r_num = right_prim.to_numeric(interpreter)?;
        debug!("Checking: {} {:?} {}", l_num, op.get_kind(), r_num);
        // assert these are the same type when doing bigints
        let result = match op.get_kind() {
            Kind::Plus => add(l_num, r_num),
            Kind::Minus => subtract(l_num, r_num),
            Kind::Star => multiply(l_num, r_num),
            Kind::Slash => divide(l_num, r_num),
            Kind::Percent => remainder(l_num, r_num),
            Kind::LessThan => {
                let res = less_than(l_num, r_num);
                return Ok(JSValue::new_boolean(&res));
            }
            Kind::LessThanOrEquals => {
                let lt = less_than(l_num, r_num);
                let eq = equal(l_num, r_num);
                let value = lt || eq;
                return Ok(JSValue::new_boolean(&value));
            }
            Kind::GreaterThan => {
                let res = less_than(r_num, l_num);
                return Ok(JSValue::new_boolean(&res));
            }
            Kind::GreaterThanOrEquals => {
                let gt = less_than(r_num, l_num);
                let eq = equal(l_num, r_num);
                let value = gt || eq;
                return Ok(JSValue::new_boolean(&value));
            }
            Kind::EqualEqualEqual => return Ok(JSValue::new_boolean(&equal(l_num, r_num))),
            _ => panic!("the disco"),
        };
        Ok(JSValue::new_number(&result))
    }
}

impl PartialEq for JSValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean { data: left }, Self::Boolean { data: right }) => left == right,
            (Self::String { data: left }, Self::String { data: right }) => left == right,
            (
                Self::Symbol {
                    id: l_id,
                    description: _,
                },
                Self::Symbol {
                    id: r_id,
                    description: _,
                },
            ) => l_id == r_id,
            (Self::Number { data: left }, Self::Number { data: right }) => left == right,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
