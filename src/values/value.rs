use core::f64;
use std::{
    mem::discriminant,
    sync::{Mutex, OnceLock},
};

use log::{debug, trace};
use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    errors::JSError,
    expr::Expr,
    global::{get_or_intern_string, get_string_from_pool},
    stmt::Stmt,
    token::Kind,
    utils::{get_function_params, remove_quotes_from_string},
    values::{
        JSResult, ObjectKind, PreferredType, add, bitwise_or, divide, equal, less_than, multiply,
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
pub enum JSValue {
    Null,
    Undefined,
    Boolean { data: bool },
    String { data: SymbolU32 },
    Symbol { id: usize, description: SymbolU32 },
    Number { data: f64 },
    BigInt,
    Object { object_id: usize, kind: ObjectKind },
}
// TODO - add identifier type

impl JSValue {
    pub fn to_primitive(
        &self,
        preferred_type: Option<PreferredType>,
        interpreter: &mut Interpreter,
    ) -> JSResult<JSValue> {
        match self {
            JSValue::Object { object_id, kind: _ } => {
                let obj = interpreter.get_object(*object_id)?;
                let res = obj.to_primitive(preferred_type.unwrap_or(PreferredType::Number))?;
                Ok(res)
            }
            _ => Ok(self.clone()),
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

    pub fn to_numeric(&self, interpreter: &mut Interpreter) -> JSResult<JSValue> {
        let prim_value = self.to_primitive(Some(PreferredType::Number), interpreter)?;
        match prim_value {
            JSValue::BigInt => todo!(),
            _ => self.to_number(interpreter),
        }
    }

    pub fn to_number(&self, interpreter: &mut Interpreter) -> JSResult<JSValue> {
        let res = match self {
            JSValue::Null => JSValue::new_number(&0.0),
            JSValue::Undefined => JSValue::new_number(&f64::NAN),
            JSValue::Boolean { data } => JSValue::new_number(match data {
                true => &1.0,
                false => &0.0,
            }),
            JSValue::String { data } => JSValue::new_number(&JSValue::string_to_number(data)),
            JSValue::Symbol {
                id: _,
                description: _,
            } => {
                return Err(JSError::new_function_type_error(
                    "Cannot convert a Symbol value to a number",
                ));
            }
            JSValue::BigInt => {
                return Err(JSError::new_function_type_error(
                    "Cannot convert a BigInt value to a number",
                ));
            }
            JSValue::Number { data: _ } => self.clone(),
            JSValue::Object { object_id, kind: _ } => {
                let object = interpreter.get_object_mut(*object_id)?;
                let prim_value = object.to_primitive(PreferredType::Number)?;
                prim_value.to_number(interpreter)?
            }
        };
        Ok(res)
    }

    pub fn string_to_number(value: &SymbolU32) -> f64 {
        let string = get_string_from_pool(value).expect("Prevented by spec");
        let number = string.parse::<f64>();
        match number {
            Ok(n) => n,
            Err(_) => f64::NAN,
        }
    }

    pub fn to_integer_or_infinity(&self, interpreter: &mut Interpreter) -> JSResult<JSValue> {
        let number = self.to_number(interpreter)?.get_number();
        if number.is_nan() || number == 0.0 {
            return Ok(JSValue::new_number(&0.0));
        }
        if number.is_infinite() {
            return Ok(JSValue::new_number(&number));
        }
        let floored = f64::floor(number);
        Ok(JSValue::new_number(&floored))
    }

    pub fn is_finite(&self, interpreter: &mut Interpreter) -> JSResult<bool> {
        let number = self.to_number(interpreter)?.get_number();
        Ok(number.is_finite())
    }

    pub fn to_int_32(&self, interpreter: &mut Interpreter) -> JSResult<i32> {
        let number = self.to_number(interpreter)?.get_number();
        let is_finite = self.is_finite(interpreter)?;
        if !is_finite || number == 0.0 || number == -0.0 {
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
        let number = self.to_number(interpreter)?.get_number();
        let is_finite = self.is_finite(interpreter)?;
        if !is_finite || number == 0.0 || number == -0.0 {
            return Ok(0);
        }
        let int = number.trunc() as u32;
        let rhs_mod = 2u32.pow(32);
        let int32bit = int % rhs_mod;
        Ok(int32bit)
    }

    pub fn to_int_16(&self, interpreter: &mut Interpreter) -> JSResult<i16> {
        let number = self.to_number(interpreter)?.get_number();
        let is_finite = self.is_finite(interpreter)?;
        if !is_finite || number == 0.0 || number == -0.0 {
            return Ok(0);
        }
        let int = number.trunc() as i16;
        let rhs_mod = 2i16.pow(16);
        let int16bit = int % rhs_mod;
        if int16bit >= 2i16.pow(15) {
            return Ok(int16bit - rhs_mod);
        }

        Ok(int16bit)
    }

    pub fn to_uint_16(&self, interpreter: &mut Interpreter) -> JSResult<u16> {
        let number = self.to_number(interpreter)?.get_number();
        let is_finite = self.is_finite(interpreter)?;
        if !is_finite || number == 0.0 || number == -0.0 {
            return Ok(0);
        }
        let int = number.trunc() as u16;
        let rhs_mod = 2u16.pow(16);
        let int16bit = int % rhs_mod;
        Ok(int16bit)
    }

    pub fn to_int_8(&self, interpreter: &mut Interpreter) -> JSResult<i8> {
        let number = self.to_number(interpreter)?.get_number();

        let is_finite = self.is_finite(interpreter)?;
        if !is_finite || number == 0.0 || number == -0.0 {
            return Ok(0);
        }
        let int = number.trunc() as i8;
        let rhs_mod = 2i8.pow(8);
        let int8bit = int % rhs_mod;
        if int8bit >= 2i8.pow(7) {
            return Ok(int8bit - rhs_mod);
        }

        Ok(int8bit)
    }

    pub fn to_uint_8(&self, interpreter: &mut Interpreter) -> JSResult<u8> {
        let number = self.to_number(interpreter)?.get_number();
        let is_finite = self.is_finite(interpreter)?;
        if !is_finite || number == 0.0 || number == -0.0 {
            return Ok(0);
        }
        let int = number.trunc() as u8;
        let rhs_mod = 2u8.pow(8);
        let int8bit = int % rhs_mod;
        Ok(int8bit)
    }

    pub fn to_uint_8_clamped(&self, interpreter: &mut Interpreter) -> JSResult<u8> {
        let number = self.to_number(interpreter)?.get_number();
        if number.is_nan() {
            return Ok(0);
        }
        let clamped = number.clamp(0.0, 255.0);
        let floor = number.floor();
        if clamped < floor + 0.5 {
            return Ok(floor as u8);
        }
        if clamped > floor + 0.5 {
            return Ok(floor as u8 + 1);
        }
        let floor = floor as u8;
        if floor % 2 == 0 {
            return Ok(floor);
        }
        Ok(floor + 1)
    }

    pub fn to_big_int(&self) -> JSResult<JSValue> {
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
            JSValue::Object { object_id, kind: _ } => {
                let object = interpreter.get_object(*object_id)?;
                let prim_value = object.to_primitive(PreferredType::String)?;
                prim_value.to_string(interpreter)?
            }
        })
    }

    pub fn to_length(&self, interpreter: &mut Interpreter) -> JSResult<JSValue> {
        let len = self.to_integer_or_infinity(interpreter)?;
        if let JSValue::Number { data } = len {
            if data <= 0.0 {
                return Ok(JSValue::new_number(&0.0));
            }
            let res = f64::min(data, (2u64.pow(53) - 1) as f64);
            return Ok(JSValue::new_number(&res));
        }
        Err(JSError::new("To integer or infinity returned not a number"))
    }

    pub fn is_undefined(&self) -> bool {
        discriminant(self) == discriminant(&JSValue::Undefined)
    }

    pub fn is_object(&self) -> bool {
        discriminant(self)
            == discriminant(&JSValue::Object {
                object_id: 0,
                kind: ObjectKind::Object,
            })
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

    pub fn is_number(&self) -> bool {
        discriminant(self) == discriminant(&JSValue::Number { data: f64::NAN })
    }

    pub fn is_null(&self) -> bool {
        discriminant(self) == discriminant(&JSValue::Null)
    }

    pub fn is_boolean(&self) -> bool {
        discriminant(self) == discriminant(&JSValue::Boolean { data: false })
    }

    pub fn is_big_int(&self) -> bool {
        discriminant(self) == discriminant(&JSValue::BigInt)
    }

    pub fn new_number(v: &f64) -> Self {
        Self::Number { data: *v }
    }

    pub fn new_boolean(v: bool) -> Self {
        Self::Boolean { data: v }
    }

    pub fn new_undefined() -> Self {
        Self::Undefined
    }

    pub fn new_null() -> Self {
        Self::Null
    }

    pub fn new_object(properties: Properties, interpreter: &mut Interpreter) -> Self {
        let object_id = JSObject::new_ordinary_object(properties, true, None, interpreter);
        Self::Object {
            object_id,
            kind: ObjectKind::Object,
        }
    }

    pub fn object_shallow_copy(id: ObjectId, kind: ObjectKind) -> Self {
        Self::Object {
            object_id: id,
            kind,
        }
    }

    pub fn get_object_id(&self) -> JSResult<usize> {
        if let JSValue::Object { object_id, kind: _ } = self {
            return Ok(*object_id);
        }
        Err(JSError::new("Object not found"))
    }

    pub fn get_object<'a>(&'a self, interpreter: &'a Interpreter) -> JSResult<&'a JSObject> {
        if let JSValue::Object { object_id, kind: _ } = self {
            return interpreter.get_object(*object_id);
        }
        Err(JSError::new("Expected object"))
    }

    pub fn new_string(s: &SymbolU32) -> Self {
        trace!("new string: {}", get_string_from_pool(s).unwrap());
        Self::String { data: *s }
    }

    pub fn new_array(properties: Properties, interpreter: &mut Interpreter) -> Self {
        let object_id = JSObject::new_array_object(properties, interpreter);
        Self::Object {
            object_id,
            kind: ObjectKind::Array,
        }
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
        let value = JSValue::Object {
            object_id,
            kind: ObjectKind::Function,
        };
        interpreter.new_variable(ident_id, false, value);

        Ok(JSValue::Undefined)
    }

    pub fn apply_string_or_numeric_binary_operator(
        &self,
        op: &Kind,
        right: &JSValue,
        interpreter: &mut Interpreter,
    ) -> JSResult<JSValue> {
        let mut l_val = self.clone();
        let mut r_val = right.clone();
        if *op == Kind::Plus {
            let left_prim = self.to_primitive(None, interpreter)?;
            let right_prim = right.to_primitive(None, interpreter)?;

            if left_prim.is_string() || right_prim.is_string() {
                let left_str_sym = left_prim.to_string(interpreter)?;
                let right_str_sym = right_prim.to_string(interpreter)?;
                let left_str = get_string_from_pool(&left_str_sym).unwrap(); // panic should be fine here, programmer error not JS error
                let right_str = get_string_from_pool(&right_str_sym).unwrap();
                // we store strings with quote marks to distinguish from identifiers
                // concatenation means stripping the quotes (if present, it is valid to concatenate an identifier too)
                // then adding them back in at the end as we know we have a string
                let left_str = remove_quotes_from_string(&left_str);
                let right_str = remove_quotes_from_string(&right_str);
                let concatenated = format!("'{left_str}{right_str}'");
                let id = get_or_intern_string(&concatenated);
                return Ok(JSValue::new_string(&id));
            }
            l_val = left_prim;
            r_val = right_prim;
        };
        // must be numbers at this point
        let l_num = l_val.to_numeric(interpreter)?.get_number();
        let r_num = r_val.to_numeric(interpreter)?.get_number();
        debug!("Checking: {} {:?} {}", l_num, op, r_num);
        // assert these are the same type when doing bigints
        let result = match op {
            Kind::Plus => add(l_num, r_num),
            Kind::Minus => subtract(l_num, r_num),
            Kind::Star => multiply(l_num, r_num),
            Kind::Slash => divide(l_num, r_num),
            Kind::Percent => remainder(l_num, r_num),
            Kind::LessThan => {
                let res = less_than(l_num, r_num);
                return Ok(JSValue::new_boolean(res));
            }
            Kind::LessThanOrEquals => {
                let lt = less_than(l_num, r_num);
                let eq = equal(l_num, r_num);
                let value = lt || eq;
                return Ok(JSValue::new_boolean(value));
            }
            Kind::GreaterThan => {
                let res = less_than(r_num, l_num);
                return Ok(JSValue::new_boolean(res));
            }
            Kind::GreaterThanOrEquals => {
                let gt = less_than(r_num, l_num);
                let eq = equal(l_num, r_num);
                let value = gt || eq;
                return Ok(JSValue::new_boolean(value));
            }
            Kind::EqualEqual => return Ok(JSValue::new_boolean(equal(l_num, r_num))),
            Kind::EqualEqualEqual => return Ok(JSValue::new_boolean(equal(l_num, r_num))),
            Kind::NotEqual => {
                let result = equal(l_num, r_num);
                return Ok(JSValue::new_boolean(!result));
            }
            Kind::BitwiseOr => {
                let result = bitwise_or(l_num, r_num, interpreter);
                result as f64
            }
            _ => panic!("the disco"),
        };
        Ok(JSValue::new_number(&result))
    }

    pub fn compute_equality(
        &self,
        operator: &Kind,
        rhs: &Self,
        interpreter: &mut Interpreter,
    ) -> JSResult<JSValue> {
        match operator {
            Kind::EqualEqual => interpreter.is_loosely_equal(self, rhs),
            Kind::EqualEqualEqual => interpreter.is_strictly_equal(self, rhs),
            Kind::NotEqual => {
                let res = interpreter.is_loosely_equal(self, rhs)?;
                let boolean = res.get_boolean();
                return Ok(JSValue::new_boolean(!boolean));
            }
            Kind::NotEqualEqual => {
                let res = interpreter.is_strictly_equal(self, rhs)?;
                let boolean = res.get_boolean();
                return Ok(JSValue::new_boolean(!boolean));
            }
            _ => panic!("Unhandled equality operator: {:?}", operator),
        }
    }

    pub fn compute_logical_and(v1: Self, v2: Self) -> JSResult<Self> {
        let b1 = v1.to_boolean();
        if !b1 {
            return Ok(v1);
        }
        Ok(v2)
    }

    pub fn compute_logical_or(v1: Self, v2: Self) -> JSResult<Self> {
        let b1 = v1.to_boolean();
        if b1 {
            return Ok(v1);
        }
        Ok(v2)
    }

    pub fn get_number(&self) -> f64 {
        if let JSValue::Number { data } = self {
            return *data;
        }
        panic!("Expected number value from calling function")
    }

    pub fn get_boolean(&self) -> bool {
        if let JSValue::Boolean { data } = self {
            return *data;
        }
        panic!("Expected boolean value from calling function")
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
