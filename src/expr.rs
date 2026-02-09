use std::fmt;

use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    errors::JSError,
    global::{get_or_intern_string, get_string_from_pool},
    stmt::Stmt,
    token::{Kind, Token},
    utils::get_function_params,
    values::{JSObject, JSResult, JSValue},
};

#[derive(Clone, Debug)]
pub enum LogKind {
    Log,
    Error,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Assignment {
        identifier: Box<Expr>,
        right: Box<Expr>,
    },
    Binary {
        operator: Kind,
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
    ObjectCall {
        identifier: Box<Expr>,
        expr: Box<Expr>,
    },
    FunctionCall {
        identifier: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Postfix {
        left: Box<Expr>,
        operator: Token,
    },
    FunctionDecl {
        identifier: Option<Box<Expr>>,
        arguments: Vec<Expr>,
        body: Box<Stmt>,
    },
    // internal only
    PrintExpr {
        kind: LogKind,
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

    pub fn new_binary(operator: Kind, left: Expr, right: Expr) -> Self {
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

    pub fn new_object_call(identifier: Expr, expr: Expr) -> Self {
        Self::ObjectCall {
            identifier: Box::new(identifier),
            expr: Box::new(expr),
        }
    }

    pub fn new_function_call(identifier: Expr, arguments: Vec<Expr>) -> Self {
        Self::FunctionCall {
            identifier: Box::new(identifier),
            arguments,
        }
    }

    pub fn new_postfix(left: Expr, operator: Token) -> Self {
        Self::Postfix {
            left: Box::new(left),
            operator,
        }
    }

    pub fn new_function_decl(
        identifier: Option<Box<Expr>>,
        arguments: Vec<Expr>,
        body: Stmt,
    ) -> Self {
        Self::FunctionDecl {
            identifier,
            arguments,
            body: Box::new(body),
        }
    }

    pub fn new_print_expr(kind: LogKind) -> Self {
        Self::PrintExpr { kind }
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
                                let obj = interpreter.get_object_mut(object_id)?;
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
            Self::Postfix { left, operator } => {
                todo!()
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
                panic!("{}", format!("Unhandled operator: {:?}", operator));
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
                        let var = interpreter.get_var(id)?;
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
                let exists = interpreter.get_value_from_environment(None, *string_index);
                // let exists = interpreter.get_variable_from_environment(None, *string_index);
                match exists {
                    Ok(value) => {
                        // already exists, so we evaluate the value
                        Ok(value.clone())
                    }
                    // doesn't exist yet, this is a declaration / initialization. pass the interned index to the statement
                    Err(_) => Ok(JSValue::new_string(string_index)),
                }
            }
            Expr::FunctionCall {
                identifier,
                arguments,
            } => {
                let args = arguments
                    .into_iter()
                    .map(|arg| {
                        let res = arg.evaluate(interpreter)?;
                        Ok(res)
                    })
                    .collect::<JSResult<Vec<JSValue>>>()?;
                // get variable out of local environment
                let idx = if let Expr::Identifier { string_index } = **identifier {
                    string_index
                } else {
                    get_or_intern_string("data")
                };

                let value = identifier.evaluate(interpreter)?;
                let object = value.get_object(interpreter)?.clone();
                let result = object.call(args, &idx, interpreter)?;
                Ok(result)
            }
            Expr::ObjectCall { identifier, expr } => {
                let object = identifier.evaluate(interpreter)?;
                let expr = expr.evaluate(interpreter)?;
                let key = expr.to_string(interpreter)?;
                if let JSValue::Object { object_id } = object {
                    let obj = interpreter.get_object(object_id).unwrap();
                    let property = obj.get_property(&key).unwrap();
                    let value = property.get_value()?.clone();
                    return Ok(value);
                }

                Err(JSError::new("Object called with invalid property"))
            } // implement function expressions and arrow functions
            Expr::FunctionDecl {
                identifier,
                arguments,
                body,
            } => {
                let ident = if let Some(id) = identifier {
                    id.evaluate(interpreter)?
                } else {
                    JSValue::Undefined
                };
                let ident_id = ident.to_string(interpreter)?;
                let scope_id = interpreter.enter_scope(None);
                let parameters = get_function_params(arguments, interpreter)?;
                for param in &parameters {
                    interpreter.new_variable(*param, true, JSValue::Undefined);
                }
                interpreter.leave_scope();
                let object_id =
                    JSObject::new_function_object(body.clone(), parameters, scope_id, interpreter);

                // function expressions return the function itself
                Ok(JSValue::Object { object_id })
            }
            Expr::PrintExpr { kind } => {
                let data = get_or_intern_string("data");
                let variable = interpreter.get_variable_from_current_environment(data);
                if let Some(var_id) = variable {
                    let var = interpreter.get_var(var_id)?;
                    let value = var.get_value_cloned();
                    let s = value.to_string(interpreter)?;
                    let s = get_string_from_pool(&s);
                    if let Some(out) = s {
                        match kind {
                            LogKind::Log => {
                                println!("{out}");
                            }
                            LogKind::Error => {
                                eprintln!("{out}");
                            }
                        }
                    }
                }

                Ok(JSValue::Undefined)
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
            Expr::Postfix { operator, left } => {
                write!(f, "Postfix({} {:?})", left, operator)
            }
            Expr::Identifier { string_index } => {
                write!(f, "Identifier({:?})", string_index)
            }
            Expr::ObjectCall { identifier, expr } => {
                write!(f, "ObjectCall {} ({})", identifier, expr)
            }
            Expr::FunctionCall {
                identifier,
                arguments,
            } => {
                let args = arguments
                    .iter()
                    .map(|arg| format!("{arg}"))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "FunctionCall {identifier}({args})")
            }
            Expr::FunctionDecl {
                identifier,
                arguments,
                body,
            } => {
                let args = arguments
                    .iter()
                    .map(|arg| format!("{arg}"))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(
                    f,
                    "FunctionDecl {}({args}) {{ {body} }}",
                    identifier
                        .clone()
                        .unwrap_or(Box::new(Expr::new_literal(JSValue::Undefined)))
                )
            }
            Expr::PrintExpr { kind } => {
                write!(f, "Console.{kind:?}",)
            }
        }
    }
}
