use std::fmt;

use crate::token::Token;

#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    String(usize),
    Boolean(bool),
}

impl Value {
    pub fn new_number(num: &f64) -> Self {
        Self::Number(*num)
    }

    pub fn new_string(idx: &usize) -> Self {
        Self::String(*idx)
    }

    pub fn new_boolean(b: &bool) -> Self {
        Self::Boolean(*b)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(v) => write!(f, "{v}"),
            Self::Number(v) => write!(f, "{v}"),
            Self::String(v) => write!(f, "index: {v}"),
        }
    }
}

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
    Variable(usize),
}

impl Expr {
    pub fn new_literal(value: Value) -> Self {
        Self::Literal(value)
    }

    pub fn new_grouping(expr: Expr) -> Self {
        Self::Grouping(Box::new(expr))
    }

    pub fn new_variable(value: &usize) -> Self {
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
                write!(f, "Variable({})", id)
            }
        }
    }
}
