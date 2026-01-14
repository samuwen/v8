use std::fmt;

use string_interner::symbol::SymbolU32;

use crate::{stmt::Stmt, token::Token};

#[derive(Clone, Debug)]
pub enum ArrowFunctionReturn {
    Expr(Box<Expr>),
    Stmt(Box<Stmt>),
}

#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    String(SymbolU32),
    Boolean(bool),
    Null,
    Undefined,
    Array(Vec<Expr>),
    Object(Vec<(SymbolU32, Expr)>),
    Function {
        identifier: Option<Box<Expr>>,
        arguments: Vec<Expr>,
        body: Box<Stmt>,
    },
    ArrowFunction {
        arguments: Vec<Expr>,
        body: ArrowFunctionReturn,
    },
}

impl Value {
    pub fn new_number(num: &f64) -> Self {
        Self::Number(*num)
    }

    pub fn new_string(idx: &SymbolU32) -> Self {
        Self::String(*idx)
    }

    pub fn new_boolean(b: &bool) -> Self {
        Self::Boolean(*b)
    }

    pub fn new_null() -> Self {
        Self::Null
    }

    pub fn new_undefined() -> Self {
        Self::Undefined
    }

    pub fn new_array(expressions: Vec<Expr>) -> Self {
        Self::Array(expressions)
    }

    pub fn new_object(pairs: Vec<(SymbolU32, Expr)>) -> Self {
        Self::Object(pairs)
    }

    pub fn new_function(identifier: Option<Expr>, args: Vec<Expr>, body: Stmt) -> Self {
        Self::Function {
            identifier: identifier.map(|id| Box::new(id)),
            arguments: args,
            body: Box::new(body),
        }
    }

    pub fn new_arrow_function(args: Vec<Expr>, body: ArrowFunctionReturn) -> Self {
        Self::ArrowFunction {
            arguments: args,
            body,
        }
    }
}

impl Value {
    fn fmt_indented(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let indent_str = "  ".repeat(indent);

        match self {
            Self::Boolean(v) => write!(f, "{v}"),
            Self::Number(v) => write!(f, "{v}"),
            Self::String(v) => write!(f, "index: {v:?}"),
            Self::Null => write!(f, "null"),
            Self::Undefined => write!(f, "undefined"),
            Self::Array(e) => write!(
                f,
                "[{}]",
                e.iter()
                    .map(|exp| exp.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Object(pairs) => write!(
                f,
                "{{ {} }}",
                pairs
                    .iter()
                    .map(|(key, value)| format!("{key:?}: {value}"))
                    .collect::<Vec<String>>()
                    .join("\n"),
            ),
            Value::Function {
                identifier,
                arguments,
                body,
            } => {
                writeln!(f, "{}FunctionExpression {{", indent_str)?;
                writeln!(f, "{}  identifier: ", indent_str,)?;
                match identifier {
                    Some(ident) => {
                        writeln!(f, "{}", ident)?;
                    }
                    None => writeln!(f, "{{}}")?,
                }
                writeln!(f, "{}  arguments: [", indent_str)?;
                for arg in arguments {
                    writeln!(f, "{}    {}", indent_str, arg)?;
                }
                writeln!(f, "{}  ]", indent_str)?;
                writeln!(f, "{}  body:", indent_str)?;
                body.fmt_indented(f, indent + 2)?;
                writeln!(f, "{}}}", indent_str)
            }
            Value::ArrowFunction { arguments, body } => {
                writeln!(f, "{}ArrowFunction {{", indent_str)?;
                writeln!(f, "{}  arguments: [", indent_str)?;
                for arg in arguments {
                    writeln!(f, "{}    {}", indent_str, arg)?;
                }
                writeln!(f, "{}  ]", indent_str)?;
                writeln!(f, "{}  body:", indent_str)?;
                match body {
                    ArrowFunctionReturn::Stmt(stmt) => stmt.fmt_indented(f, indent + 2)?,
                    ArrowFunctionReturn::Expr(expr) => writeln!(f, "{}    {}", indent, expr)?,
                }
                writeln!(f, "{}}}", indent_str)
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_indented(f, 0)
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
