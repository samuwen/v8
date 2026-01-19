use std::fmt;

use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    span::Span,
    token::{Kind, Token},
    values::JSValue,
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
    Variable {
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

    pub fn new_variable(value: &SymbolU32, span: Span) -> Self {
        Self::Variable {
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

    pub fn evaluate(&self, interpreter: &mut Interpreter) -> JSValue {
        match self {
            Self::Literal { value, span: _ } => value.clone(),
            Self::Unary {
                operator,
                right,
                span: _,
            } => {
                let right = right.evaluate(interpreter);
                match operator.get_kind() {
                    Kind::Bang => {
                        let val_as_bool = right.to_boolean();
                        let negated = !val_as_bool;
                        JSValue::new_boolean(&negated)
                    }
                    // Kind::Minus => {
                    //     let val_as_number = right.convert_to_number();
                    //     Value::Number(-val_as_number)
                    // }
                    _ => panic!("Invalid unary operation: {:?}", operator.get_kind()),
                }
            }
            Self::Binary {
                operator,
                left,
                right,
                span,
            } => {
                let left = left.evaluate(interpreter);
                let right = right.evaluate(interpreter);
                // if left.is_same_variant(&right) {
                //     match operator.get_kind() {
                //         _ => panic!("fuck javascript"),
                //     }
                // }
                panic!("Fuck javascript gdi")
            }
            _ => panic!("the disco"),
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

            Expr::Variable {
                string_index,
                span: _,
            } => {
                write!(f, "Variable({:?})", string_index)
            }
        }
    }
}
