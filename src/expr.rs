use std::fmt;

use string_interner::symbol::SymbolU32;

use crate::{
    Interpreter,
    errors::JSError,
    global::{get_or_intern_string, get_string_from_pool},
    stmt::Stmt,
    token::{Kind, Token},
    utils::get_function_params,
    values::{JSObject, JSResult, JSValue, ObjectKind},
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

    pub fn new_object_call(expr: Expr, identifier: Expr) -> Self {
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
                        Ok(JSValue::new_boolean(negated))
                    }
                    Kind::Minus => {
                        let val_as_number = right.to_numeric(interpreter)?.get_number();
                        Ok(JSValue::new_number(&-val_as_number))
                    }
                    Kind::Plus => {
                        let val_as_number = right.to_numeric(interpreter)?.get_number();
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
                            JSValue::Object { object_id, kind: _ } => {
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
            Self::Postfix {
                left: _,
                operator: _,
            } => {
                todo!()
            }
            Self::Binary {
                operator,
                left,
                right,
            } => {
                let left = left.evaluate(interpreter)?;
                let right = right.evaluate(interpreter)?;
                if operator.is_equality_operator() {
                    return left.compute_equality(operator, &right, interpreter);
                }
                if operator.is_binary_operator() {
                    return left.apply_string_or_numeric_binary_operator(
                        operator,
                        &right,
                        interpreter,
                    );
                }
                if *operator == Kind::LogicalOr {
                    return JSValue::compute_logical_or(left, right);
                }
                if *operator == Kind::LogicalAnd {
                    return JSValue::compute_logical_and(left, right);
                }
                panic!("{}", format!("Unhandled operator: {:?}", operator));
            }
            Expr::Assignment { identifier, right } => {
                let rhs = right.evaluate(interpreter)?;
                if let Expr::ObjectCall { identifier, expr } = &**identifier {
                    let expr = expr.evaluate(interpreter)?;
                    if let JSValue::Object { object_id, kind } = expr {
                        let identifier = identifier.evaluate(interpreter)?; // accessor
                        let key = identifier.to_string(interpreter)?;

                        let object = interpreter.get_object_mut(object_id)?;
                        let prop = object.get_property_mut(&key);
                        if let Some(prop) = prop {
                            prop.set_value(rhs);
                            let value = prop.get_value()?;
                            return Ok(value.clone());
                        }
                    }
                }
                let ident_index = if let Expr::Identifier { string_index } = **identifier {
                    string_index
                } else {
                    return Err(JSError::new("Invalid left-hand side in assignment"));
                };
                let exists = interpreter.get_variable_from_current_environment(ident_index);
                match exists {
                    Ok(var) => {
                        if var.is_mutable() {
                            var.update_value(rhs.clone())?;
                            return Ok(rhs);
                        } else {
                            return Err(JSError::new(
                                "Syntax error: Cannot assign to constant variable",
                            ));
                        }
                    }
                    Err(_) => Ok(JSValue::new_string(&ident_index)),
                }
            }
            Expr::Grouping { expr } => Ok(expr.evaluate(interpreter)?),
            Expr::Identifier { string_index } => {
                let exists = interpreter.get_value_from_environment(*string_index);
                match exists {
                    Ok(val) => Ok(val.clone()),
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
                let value = identifier.evaluate(interpreter)?;
                match value {
                    JSValue::String { data: ident_index } => {
                        let value = interpreter.get_value_from_environment(ident_index)?.clone();
                        let object = value.get_object(interpreter)?.clone();
                        let result = object.call(args, Some(&ident_index), interpreter)?;
                        Ok(result)
                    }
                    JSValue::Object { object_id, kind } => {
                        if let ObjectKind::Function = kind {
                            let obj = interpreter.get_object(object_id)?.clone();
                            let result = obj.call(args, None, interpreter)?;
                            return Ok(result);
                        }
                        panic!("Attempting to call an ordinary object")
                    }
                    _ => panic!("Attempting to call something that should not be called"),
                }
                // let ident_index = value.to_string(interpreter)?;
                // let value = interpreter
                //     .get_value_from_environment(None, ident_index)?
                //     .clone();
                // let object = value.get_object(interpreter)?.clone();
                // let result = object.call(args, &ident_index, interpreter)?;
                // Ok(result)
            }
            Expr::ObjectCall { identifier, expr } => {
                let expr = expr.evaluate(interpreter)?;
                let key = expr.to_string(interpreter)?;
                let ident_res = identifier.evaluate(interpreter)?;
                match ident_res {
                    JSValue::Object { object_id, kind: _ } => {
                        let object = interpreter.get_object(object_id)?;
                        let property = object.get_property(&key).unwrap();
                        let value = property.get_value()?.clone();
                        return Ok(value);
                    }
                    JSValue::String { data: ident } => {
                        let value = interpreter.get_value_from_environment(ident);
                        let object = match value {
                            Ok(v) => v,
                            Err(_) => {
                                let string_value =
                                    get_string_from_pool(&ident).expect("Uninitialized string");
                                return Err(JSError::new(&format!(
                                    "Unitialized variable: {string_value}"
                                )));
                            }
                        };
                        if let JSValue::Object { object_id, kind: _ } = *object {
                            let obj = interpreter.get_object(object_id).unwrap();
                            let property = obj.get_property(&key).unwrap();
                            let value = property.get_value()?.clone();
                            return Ok(value);
                        }
                    }
                    JSValue::Number { data: _ } => {
                        if let JSValue::Object { object_id, kind: _ } = expr {
                            let ident = ident_res.to_string(interpreter)?;
                            let object = interpreter.get_object(object_id)?;
                            let property = object.get_property(&ident);
                            if let Some(property) = property {
                                let value = property.get_value()?;
                                return Ok(value.clone());
                            }
                        }
                    }
                    _ => {
                        println!("{ident_res:?}");
                        println!("{expr:?}");
                        unimplemented!()
                    }
                }

                Err(JSError::new("Object called with invalid property"))
            }
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

                let object_val = JSValue::Object {
                    object_id,
                    kind: ObjectKind::Function,
                };
                interpreter.new_variable(ident_id, false, object_val.clone());

                // function expressions return the function itself
                Ok(object_val)
            }
            Expr::PrintExpr { kind } => {
                let data = get_or_intern_string("data");
                let variable = interpreter.get_variable_from_current_environment(data);
                if let Ok(var) = variable {
                    let value = var.get_value_cloned();
                    let s = value.to_string(interpreter)?;
                    let maybe_val = interpreter.get_value_from_environment(s);
                    match maybe_val {
                        Ok(val) => {
                            let value = val.clone().to_string(interpreter)?;
                            let string = get_string_from_pool(&value);
                            if let Some(out) = string {
                                add_message(&out, kind, interpreter);
                            }
                        }
                        Err(_) => {
                            let s = get_string_from_pool(&s);
                            if let Some(out) = s {
                                add_message(&out, kind, interpreter);
                            }
                        }
                    }
                }

                Ok(JSValue::Undefined)
            }
        }
    }
}

fn add_message(message: &str, kind: &LogKind, interpreter: &mut Interpreter) {
    let quote = '\'';
    let len = message.len();
    let message = if message.starts_with(quote) && message.ends_with(quote) {
        &message[1..len - 1]
    } else {
        &message
    };
    let message = format!("{message}\n");
    match kind {
        LogKind::Log => {
            interpreter.output_buffer.push_str(&message);
        }
        LogKind::Error => {
            interpreter.error_buffer.push_str(&message);
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
