use std::fmt;

use string_interner::symbol::SymbolU32;

use crate::{
    global::SharedContext,
    token::{Kind, Token},
    value::Value,
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
    Grouping(Box<Expr>),
    Literal(Value),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable(SymbolU32),
}

impl Expr {
    pub fn new_literal(value: Value) -> Self {
        Self::Literal(value)
    }

    pub fn new_grouping(expr: Expr) -> Self {
        Self::Grouping(Box::new(expr))
    }

    pub fn new_variable(value: &SymbolU32) -> Self {
        Self::Variable(*value)
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

    pub fn evaluate(&self, context: &mut SharedContext) -> Value {
        match self {
            Self::Literal(value) => value.clone(),
            Self::Unary { operator, right } => {
                let right = right.evaluate(context);
                match operator.get_kind() {
                    Kind::Bang => {
                        let val_as_bool = right.convert_to_boolean(context);
                        Value::Boolean(!val_as_bool)
                    }
                    Kind::Minus => {
                        let val_as_number = right.convert_to_number(context);
                        Value::Number(-val_as_number)
                    }
                    _ => panic!("Invalid unary operation: {:?}", operator.get_kind()),
                }
            }
            Self::Binary {
                operator,
                left,
                right,
            } => {
                let left = left.evaluate(context);
                let right = right.evaluate(context);
                panic!("Fuck javascript gdi")
            }
            _ => panic!("the disco"),
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

            Expr::Grouping(expr) => {
                write!(f, "Grouping({})", expr)
            }

            Expr::Literal(value) => {
                write!(f, "Literal({})", value)
            }

            Expr::Unary { operator, right } => {
                write!(f, "Unary({:?} {})", operator, right)
            }

            Expr::Variable(id) => {
                write!(f, "Variable({:?})", id)
            }
        }
    }
}
