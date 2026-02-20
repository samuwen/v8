use std::fmt;

use log::info;

use crate::{
    Interpreter,
    errors::{ErrorKind, JSError},
    expr::Expr,
    global::get_string_from_pool,
    utils::get_function_params,
    values::{JSObject, JSResult, JSValue, ObjectKind},
};

#[derive(Clone, Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Break,
    Continue,
    Expression(Box<Expr>),
    For {
        initializer: Option<Box<Stmt>>,
        condition: Option<Expr>,
        state: Option<Expr>,
        body: Box<Stmt>,
    },
    FunctionDecl {
        identifier: Box<Expr>,
        arguments: Vec<Expr>,
        body: Box<Stmt>,
    },
    If {
        condition: Box<Expr>,
        branch_true: Box<Stmt>,
        branch_false: Option<Box<Stmt>>,
    },
    Return(Option<Expr>),
    VariableDecl {
        is_mutable: bool,
        identifier: Box<Expr>,
        initializer: Option<Expr>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Stmt>,
    },
}

impl Stmt {
    pub fn new_block(stmts: Vec<Stmt>) -> Self {
        Self::Block(stmts)
    }

    pub fn new_return(expr: Option<Expr>) -> Self {
        Self::Return(expr)
    }

    pub fn new_expression(expr: Expr) -> Self {
        Self::Expression(Box::new(expr))
    }

    pub fn new_if(condition: Expr, branch_true: Stmt, branch_false: Option<Stmt>) -> Self {
        Self::If {
            condition: Box::new(condition),
            branch_true: Box::new(branch_true),
            branch_false: branch_false.map(|b| Box::new(b)),
        }
    }

    pub fn new_while(condition: Expr, body: Stmt) -> Self {
        Self::While {
            condition: Box::new(condition),
            body: Box::new(body),
        }
    }

    pub fn new_variable(is_mutable: bool, ident: Expr, initializer: Option<Expr>) -> Self {
        Self::VariableDecl {
            is_mutable,
            identifier: Box::new(ident),
            initializer,
        }
    }

    pub fn new_function(ident: Expr, args: Vec<Expr>, body: Stmt) -> Self {
        Self::FunctionDecl {
            identifier: Box::new(ident),
            arguments: args,
            body: Box::new(body),
        }
    }

    pub fn new_for(
        init: Option<Stmt>,
        cond: Option<Expr>,
        state: Option<Expr>,
        body: Stmt,
    ) -> Self {
        Self::For {
            initializer: init.map(|o| Box::new(o)),
            condition: cond,
            state,
            body: Box::new(body),
        }
    }

    pub fn evaluate(&self, interpreter: &mut Interpreter) -> JSResult<JSValue> {
        match self {
            Stmt::Block(stmts) => {
                interpreter.enter_scope(None);
                for stmt in stmts {
                    let res = stmt.evaluate(interpreter)?;
                    info!("statement result: {res:?}");
                }
                interpreter.leave_scope();
                Ok(JSValue::Undefined)
            }
            Stmt::Break => Err(JSError::new_break()),
            Stmt::Continue => Err(JSError::new_continue()),
            Self::Expression(expr) => expr.evaluate(interpreter),
            Stmt::For {
                initializer,
                condition,
                state,
                body,
            } => {
                interpreter.enter_scope(None);
                if let Some(stmt) = initializer {
                    stmt.evaluate(interpreter)?;
                }
                let mut abort_count = 0;
                'forst: loop {
                    if abort_count > 100 {
                        panic!("infinite loop")
                    }
                    if let Some(expr) = condition {
                        let value = expr.evaluate(interpreter)?;
                        if !value.to_boolean() {
                            break 'forst;
                        }
                    }
                    let body_res = body.evaluate(interpreter);
                    if let Some(expr) = state {
                        expr.evaluate(interpreter)?;
                    }

                    if let Err(e) = body_res {
                        if e.kind == ErrorKind::Break {
                            break;
                        } else if e.kind == ErrorKind::Continue {
                            continue;
                        }
                    }
                    abort_count += 1;
                }

                interpreter.leave_scope();
                Ok(JSValue::Undefined)
            }
            Stmt::FunctionDecl {
                identifier,
                arguments,
                body,
            } => {
                let ident = identifier.evaluate(interpreter)?;
                let ident_id = ident.to_string(interpreter)?;
                let scope_id = interpreter.enter_scope(None);
                let parameters = get_function_params(arguments, interpreter)?;
                for param in &parameters {
                    interpreter.new_variable(*param, true, JSValue::Undefined);
                }
                interpreter.leave_scope();
                let object_id =
                    JSObject::new_function_object(body.clone(), parameters, scope_id, interpreter);

                let value = JSValue::Object {
                    object_id,
                    kind: ObjectKind::Function,
                };
                interpreter.new_variable(ident_id, false, value);

                Ok(JSValue::Undefined)
            }
            Stmt::If {
                condition,
                branch_true,
                branch_false,
            } => {
                let evaluated_condition = condition.evaluate(interpreter)?;
                interpreter.enter_scope(None);
                if evaluated_condition.to_boolean() {
                    let b_true = branch_true.evaluate(interpreter)?;
                    return Ok(b_true);
                } else if let Some(branch_false) = branch_false {
                    let b_false = branch_false.evaluate(interpreter)?;
                    return Ok(b_false);
                }
                interpreter.leave_scope();
                Ok(JSValue::Undefined)
            }
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    let res = expr.evaluate(interpreter)?;
                    let id = interpreter.add_value(res);
                    let ret = JSError::new_return(id);
                    return Err(ret);
                }
                // hacky
                let id = interpreter.add_value(JSValue::new_undefined());
                return Err(JSError::new_return(id));
            }
            Stmt::VariableDecl {
                is_mutable,
                identifier,
                initializer,
            } => {
                // establish the variable name
                let string_index = if let Expr::Identifier { string_index } = &**identifier {
                    let already_exists =
                        interpreter.does_local_environment_already_have_variable(string_index);
                    if already_exists {
                        let kind = if *is_mutable { "let" } else { "const" };
                        let name = get_string_from_pool(string_index).unwrap(); // we know it already exists
                        return Err(JSError::new(&format!(
                            "SyntaxError: redeclaration of {kind} {name}"
                        )));
                    }
                    string_index
                } else {
                    return Err(JSError::new("Identifier expected"));
                };
                // right hand side is either the expr evaluation or undefined
                let rhs = match initializer {
                    Some(init_expr) => init_expr.evaluate(interpreter)?,
                    None => {
                        // uninitialized const is a syntax error
                        if !*is_mutable {
                            let error = JSError::new(
                                "Uncaught SyntaxError: Missing initializer in const declaration",
                            );
                            return Err(error);
                        }
                        JSValue::Undefined
                    }
                };
                // add a new variable to the variable heap
                interpreter.new_variable(*string_index, *is_mutable, rhs);

                Ok(JSValue::Undefined)
            }
            Stmt::While {
                condition: raw_condition,
                body,
            } => {
                'whilst: loop {
                    let condition = raw_condition.evaluate(interpreter)?;
                    if !condition.to_boolean() {
                        break 'whilst;
                    }
                    let body_res = body.evaluate(interpreter);
                    if let Err(e) = body_res {
                        if e.kind == ErrorKind::Break {
                            break;
                        } else if e.kind == ErrorKind::Continue {
                            continue;
                        }
                    }
                }
                Ok(JSValue::Undefined)
            }
        }
    }
}

// pretty printing
impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_indented(f, 0)
    }
}

impl Stmt {
    pub fn fmt_indented(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let indent_str = "  ".repeat(indent);

        match self {
            Stmt::Block(stmts) => {
                writeln!(f, "{}Block {{", indent_str)?;
                for stmt in stmts {
                    stmt.fmt_indented(f, indent + 1)?;
                }
                writeln!(f, "{}}}", indent_str)
            }

            Stmt::Break => {
                writeln!(f, "{}Break", indent_str)
            }

            Stmt::Continue => {
                writeln!(f, "{}Continue", indent_str)
            }

            Stmt::Expression(expr) => {
                writeln!(f, "{}Expression({})", indent_str, expr)
            }

            Stmt::For {
                initializer,
                condition,
                state,
                body,
            } => {
                writeln!(f, "{}For {{", indent_str)?;

                write!(f, "{}  initializer: ", indent_str)?;
                match initializer {
                    Some(init) => {
                        writeln!(f)?;
                        init.fmt_indented(f, indent + 2)?;
                    }
                    None => writeln!(f, "{{}}")?,
                }

                write!(f, "{}  condition: ", indent_str)?;
                match condition {
                    Some(cond) => writeln!(f, "{}", cond)?,
                    None => writeln!(f, "{{}}")?,
                }

                write!(f, "{}  state: ", indent_str)?;
                match state {
                    Some(st) => writeln!(f, "{}", st)?,
                    None => writeln!(f, "{{}}")?,
                }

                body.fmt_indented(f, indent + 2)?;

                writeln!(f, "{}}}", indent_str)
            }

            Stmt::FunctionDecl {
                identifier,
                arguments,
                body,
            } => {
                writeln!(f, "{}FunctionDecl {{", indent_str)?;
                writeln!(f, "{}  identifier: {}", indent_str, identifier)?;
                writeln!(f, "{}  arguments: [", indent_str)?;
                for arg in arguments {
                    writeln!(f, "{}    {}", indent_str, arg)?;
                }
                writeln!(f, "{}  ]", indent_str)?;
                writeln!(f, "{}  body:", indent_str)?;
                body.fmt_indented(f, indent + 2)?;
                writeln!(f, "{}}}", indent_str)
            }

            Stmt::If {
                condition,
                branch_true,
                branch_false,
            } => {
                writeln!(f, "{}If {{", indent_str)?;
                writeln!(f, "{}  condition: {}", indent_str, condition)?;
                writeln!(f, "{}  branch_true:", indent_str)?;
                branch_true.fmt_indented(f, indent + 2)?;

                write!(f, "{}  branch_false: ", indent_str)?;
                match branch_false {
                    Some(branch) => {
                        writeln!(f)?;
                        branch.fmt_indented(f, indent + 2)?;
                    }
                    None => writeln!(f, "{{}}")?,
                }

                writeln!(f, "{}}}", indent_str)
            }

            Stmt::Return(expr) => {
                write!(f, "{}Return(", indent_str)?;
                match expr {
                    Some(e) => write!(f, "{}", e)?,
                    None => write!(f, "{{}}")?,
                }
                writeln!(f, ")")
            }

            Stmt::VariableDecl {
                is_mutable,
                identifier,
                initializer,
            } => {
                writeln!(f, "{}VariableDecl {{", indent_str)?;
                writeln!(f, "{}  is_mutable: {}", indent_str, is_mutable)?;
                writeln!(f, "{}  identifier: {}", indent_str, identifier)?;
                write!(f, "{}  initializer: ", indent_str)?;
                match initializer {
                    Some(init) => writeln!(f, "{}", init)?,
                    None => writeln!(f, "{{}}")?,
                }
                writeln!(f, "{}}}", indent_str)
            }

            Stmt::While { condition, body } => {
                writeln!(f, "{}While {{", indent_str)?;
                writeln!(f, "{}  condition: {}", indent_str, condition)?;
                writeln!(f, "{}  body:", indent_str)?;
                body.fmt_indented(f, indent + 2)?;
                writeln!(f, "{}}}", indent_str)
            }
        }
    }
}
