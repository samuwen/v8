use std::fmt;

use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    errors::JSError,
    global::get_or_intern_string,
    token::{Kind, Token},
    values::{JSResult, JSValue},
};

#[derive(Clone, Debug)]
pub enum Expr {
    Assignment {
        identifier: Box<Expr>,
        right: Box<Expr>,
    },
    Binary {
        operator: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {
        value: JSValue,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Identifier {
        string_index: SymbolU32,
    },
}

impl Expr {
    pub fn new_literal(value: JSValue) -> Self {
        Self::Literal { value }
    }

    pub fn new_grouping(expr: Expr) -> Self {
        Self::Grouping {
            expr: Box::new(expr),
        }
    }

    pub fn new_identifier(value: &SymbolU32) -> Self {
        Self::Identifier {
            string_index: *value,
        }
    }

    pub fn new_unary(operator: Token, right: Expr) -> Self {
        Self::Unary {
            operator,
            right: Box::new(right),
        }
    }

    pub fn new_binary(operator: Token, left: Expr, right: Expr) -> Self {
        Self::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn new_assignment(identifier: Expr, right: Expr) -> Self {
        Self::Assignment {
            identifier: Box::new(identifier),
            right: Box::new(right),
        }
    }

    pub fn evaluate(&self, interpreter: &mut Interpreter) -> JSResult<JSValue> {
        match self {
            Self::Literal { value } => Ok(value.clone()),
            Self::Unary { operator, right } => {
                let right = right.evaluate(interpreter)?;
                match operator.get_kind() {
                    Kind::Bang => {
                        let val_as_bool = right.to_boolean();
                        let negated = !val_as_bool;
                        Ok(JSValue::new_boolean(&negated))
                    }
                    Kind::Minus => {
                        let val_as_number = right.to_numeric(interpreter)?;
                        Ok(JSValue::new_number(&-val_as_number))
                    }
                    Kind::Plus => {
                        let val_as_number = right.to_numeric(interpreter)?;
                        Ok(JSValue::new_number(&val_as_number))
                    }
                    Kind::Void => Ok(JSValue::new_undefined()),
                    Kind::Typeof => {
                        let output = match right {
                            JSValue::Null => "object",
                            JSValue::Undefined => "undefined",
                            JSValue::Boolean { data: _ } => "boolean",
                            JSValue::String { data: _ } => "string",
                            JSValue::Symbol {
                                id: _,
                                description: _,
                            } => "symbol",
                            JSValue::Number { data: _ } => "number",
                            JSValue::BigInt => "bigint",
                            JSValue::Object { object_id } => {
                                let obj = interpreter.object_heap.get_mut(object_id);
                                match obj.is_function() {
                                    true => "function",
                                    false => "object",
                                }
                            }
                        };
                        let result = get_or_intern_string(output);
                        Ok(JSValue::String { data: result })
                    }
                    _ => panic!("Invalid unary operation: {:?}", operator.get_kind()),
                }
            }
            Self::Binary {
                operator,
                left,
                right,
            } => {
                let left = left.evaluate(interpreter)?;
                let right = right.evaluate(interpreter)?;
                if operator.is_binary_operator() {
                    return left.apply_string_or_numeric_binary_operator(
                        operator,
                        &right,
                        interpreter,
                    );
                }
                panic!(
                    "{}",
                    format!("Unhandled operator: {:?}", operator.get_kind())
                );
            }
            Expr::Assignment { identifier, right } => {
                let ident_index = if let Expr::Identifier { string_index } = &**identifier {
                    string_index
                } else {
                    panic!("Assignment got passed a non-identifier identifier"); // programming error
                };
                let rhs = right.evaluate(interpreter)?;
                let exists = interpreter.get_variable_from_current_environment(*ident_index);
                match exists {
                    Some(id) => {
                        let var = interpreter.get_variable_from_heap(id);
                        if var.is_mutable() {
                            var.update_value(rhs.clone())?;
                            return Ok(rhs);
                        } else {
                            return Err(JSError::new(
                                "Syntax error: Cannot assign to constant variable",
                            ));
                        }
                    }
                    None => {
                        return Err(JSError::new("Syntax error: Variable is not initialized"));
                    }
                }
            }
            Expr::Grouping { expr } => Ok(expr.evaluate(interpreter)?),
            Expr::Identifier { string_index } => {
                let exists = interpreter.get_variable_from_current_environment(*string_index);
                match exists {
                    Some(id) => {
                        // already exists, so we evaluate the value
                        let var = interpreter.variable_heap.get_item_from_id(id);
                        Ok(var.get_value())
                    }
                    // doesn't exist yet, this is a declaration / initialization. pass the interned index to the statement
                    None => Ok(JSValue::new_string(string_index)),
                }
            }
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Assignment { identifier, right } => {
                write!(f, "Assignment({} = {})", identifier, right)
            }

            Expr::Binary {
                operator,
                left,
                right,
            } => {
                write!(f, "Binary({}, {:?}, {})", left, operator, right)
            }

            Expr::Grouping { expr } => {
                write!(f, "Grouping({})", expr)
            }

            Expr::Literal { value } => {
                write!(f, "Literal({:?})", value)
            }

            Expr::Unary { operator, right } => {
                write!(f, "Unary({:?} {})", operator, right)
            }

            Expr::Identifier { string_index } => {
                write!(f, "Identifier({:?})", string_index)
            }
        }
    }
}
