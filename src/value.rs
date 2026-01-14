use std::{fmt, ops::Add};

use string_interner::symbol::SymbolU32;

use crate::{expr::Expr, global::SharedContext, stmt::Stmt};

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

    pub fn convert_to_boolean(&self, context: &mut SharedContext) -> bool {
        match self {
            Value::Number(num) => *num == 0.0,
            Value::String(symbol) => {
                let string = context.get_string_from_pool(*symbol);
                return string != "";
            }
            Value::Boolean(b) => *b,
            Value::Null | Value::Undefined => false,
            Value::Array(_)
            | Value::Object(_)
            | Value::Function {
                identifier: _,
                arguments: _,
                body: _,
            }
            | Value::ArrowFunction {
                arguments: _,
                body: _,
            } => true,
        }
    }

    pub fn convert_to_number(&self, context: &mut SharedContext) -> f64 {
        match self {
            Value::Number(num) => *num,
            Value::String(symbol) => {
                let string = context.get_string_from_pool(*symbol);
                if string == "" {
                    return 0.0;
                } else {
                    return f64::NAN;
                }
            }
            Value::Boolean(b) => {
                if *b {
                    return 1.0;
                } else {
                    return 0.0;
                }
            }
            Value::Null | Value::Array(_) => 0.0,
            Value::Undefined
            | Value::Object(_)
            | Value::Function {
                identifier: _,
                arguments: _,
                body: _,
            }
            | Value::ArrowFunction {
                arguments: _,
                body: _,
            } => f64::NAN,
        }
    }

    pub fn convert_to_string(&self, context: &mut SharedContext) -> String {
        match self {
            Value::Number(num) => num.to_string(),
            Value::String(symbol) => context.get_string_from_pool(*symbol),
            Value::Boolean(b) => b.to_string(),
            Value::Null => String::from("null"),
            Value::Undefined => String::from("undefined"),
            Value::Array(exprs) => exprs
                .iter()
                .map(|expr| expr.evaluate(context))
                .map(|value| value.to_string())
                .collect::<Vec<String>>()
                .join(","),
            Value::Object(_) => String::from("[object Object]"),
            Value::Function {
                identifier,
                arguments,
                body,
            } => todo!(),
            Value::ArrowFunction { arguments, body } => todo!(),
        }
    }
}

// Pretty print stuff
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
