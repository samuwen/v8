use std::fmt;

use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    errors::JSError,
    global::get_or_intern_string,
    span::Span,
    token::{Kind, Token},
    values::{JSResult, JSValue},
};

#[derive(Clone, Debug)]
pub enum Expr {
    Assignment {
        identifier: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Binary {
        operator: Token,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Grouping {
        expr: Box<Expr>,
        span: Span,
    },
    Literal {
        value: JSValue,
        span: Span,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
        span: Span,
    },
    Identifier {
        string_index: SymbolU32,
        span: Span,
    },
}

impl Expr {
    pub fn new_literal(value: JSValue, span: Span) -> Self {
        Self::Literal { value, span }
    }

    pub fn new_grouping(expr: Expr, span: Span) -> Self {
        Self::Grouping {
            expr: Box::new(expr),
            span,
        }
    }

    pub fn new_identifier(value: &SymbolU32, span: Span) -> Self {
        Self::Identifier {
            string_index: *value,
            span,
        }
    }

    pub fn new_unary(operator: Token, right: Expr, span: Span) -> Self {
        Self::Unary {
            operator,
            right: Box::new(right),
            span,
        }
    }

    pub fn new_binary(operator: Token, left: Expr, right: Expr, span: Span) -> Self {
        Self::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
            span,
        }
    }

    pub fn new_assignment(identifier: Expr, right: Expr, span: Span) -> Self {
        Self::Assignment {
            identifier: Box::new(identifier),
            right: Box::new(right),
            span,
        }
    }

    pub fn evaluate(&self, interpreter: &mut Interpreter) -> JSResult<JSValue> {
        match self {
            Self::Literal { value, span: _ } => Ok(value.clone()),
            Self::Unary {
                operator,
                right,
                span: _,
            } => {
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
                span: _,
            } => {
                let left = left.evaluate(interpreter)?;
                let right = right.evaluate(interpreter)?;
                match operator.get_kind() {
                    Kind::Plus | Kind::Minus | Kind::Slash | Kind::Star | Kind::Percent => {
                        return left.apply_string_or_numeric_binary_operator(
                            operator,
                            &right,
                            interpreter,
                        );
                    }
                    _ => panic!(
                        "{}",
                        format!("Unhandled operator: {:?}", operator.get_kind())
                    ),
                }
            }
            Expr::Assignment {
                identifier,
                right,
                span: _,
            } => {
                let ident_index = if let Expr::Identifier { string_index, span } = &**identifier {
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
            Expr::Grouping { expr, span: _ } => Ok(expr.evaluate(interpreter)?),
            Expr::Identifier {
                string_index,
                span: _,
            } => {
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
            Expr::Assignment {
                identifier,
                right,
                span: _,
            } => {
                write!(f, "Assignment({} = {})", identifier, right)
            }

            Expr::Binary {
                operator,
                left,
                right,
                span: _,
            } => {
                write!(f, "Binary({}, {:?}, {})", left, operator, right)
            }

            Expr::Grouping { expr, span: _ } => {
                write!(f, "Grouping({})", expr)
            }

            Expr::Literal { value, span: _ } => {
                write!(f, "Literal({:?})", value)
            }

            Expr::Unary {
                operator,
                right,
                span: _,
            } => {
                write!(f, "Unary({:?} {})", operator, right)
            }

            Expr::Identifier {
                string_index,
                span: _,
            } => {
                write!(f, "Identifier({:?})", string_index)
            }
        }
    }
}
