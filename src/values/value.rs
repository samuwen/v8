use std::{
    mem::discriminant,
    sync::{Mutex, OnceLock},
};

use string_interner::symbol::SymbolU32;

use crate::{
    completion_record::CompletionRecord,
    errors::JSError,
    expr::Expr,
    global::get_string_from_pool,
    stmt::Stmt,
    values::{JSResult, PreferredType, objects::JSObject},
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
    Object(JSObject),
}

impl JSValue {
    pub fn to_primitive(&self, preferred_type: Option<PreferredType>) -> JSResult<JSValue> {
        // if let JSValue::Object(obj) = self {
        //     return obj.to_primitive(preferred_type);
        // }

        // Ok(self.clone())
        todo!()
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

    pub fn to_numeric(&self) -> CompletionRecord {
        todo!()
    }

    pub fn is_undefined(&self) -> bool {
        discriminant(self) == discriminant(&JSValue::Undefined)
    }

    pub fn is_object(&self) -> bool {
        match self {
            Self::Object(_) => true,
            _ => false,
        }
    }

    pub fn new_number(v: &f64) -> Self {
        todo!()
    }

    pub fn new_boolean(v: &bool) -> Self {
        todo!();
    }

    pub fn new_undefined() -> Self {
        todo!()
    }

    pub fn new_null() -> Self {
        todo!()
    }

    pub fn new_object(pairs: Vec<(SymbolU32, Expr)>) -> Self {
        todo!()
    }

    pub fn new_string(s: &SymbolU32) -> Self {
        todo!()
    }

    pub fn new_arrow_function(args: Vec<Expr>, body: ArrowFunctionReturn) -> Self {
        todo!()
    }

    pub fn new_array(args: Vec<Expr>) -> Self {
        todo!()
    }

    pub fn new_function(ident: Option<Expr>, args: Vec<Expr>, body: Stmt) -> Self {
        todo!()
    }

    pub fn as_object(&self) -> Option<JSObject> {
        // if let Self::Object(obj) = self {
        //     return Some(obj.clone());
        // }
        // None
        todo!()
    }

    pub fn call(&self, v: JSValue, args: Vec<JSValue>) -> JSResult<JSValue> {
        // if let Some(obj) = self.as_object() {
        //     if obj.is_callable() {
        //         return obj.call(v, args);
        //     }
        // }
        // let error = JSError::new_type_error("unknown");
        // return Err(error);
        todo!()
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
