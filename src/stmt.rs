use std::fmt;

use env_logger::init;

use crate::{
    Interpreter,
    errors::{ErrorKind, JSError},
    expr::Expr,
    values::{JSResult, JSValue},
    variable::Variable,
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
                for stmt in stmts {
                    stmt.evaluate(interpreter)?;
                }
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
                let mut entered_scope = false;
                if let Some(stmt) = initializer {
                    interpreter.enter_scope();
                    entered_scope = true;
                    stmt.evaluate(interpreter)?;
                }
                'forst: loop {
                    if let Some(expr) = condition {
                        let value = expr.evaluate(interpreter)?;
                        if !value.to_boolean() {
                            break 'forst;
                        }
                    }
                    if let Some(expr) = state {
                        expr.evaluate(interpreter)?;
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

                if entered_scope {
                    interpreter.leave_scope();
                }
                Ok(JSValue::Undefined)
            }
            Stmt::FunctionDecl {
                identifier,
                arguments,
                body,
            } => todo!(),
            Stmt::If {
                condition,
                branch_true,
                branch_false,
            } => {
                let evaluated_condition = condition.evaluate(interpreter)?;
                if evaluated_condition.to_boolean() {
                    let b_true = branch_true.evaluate(interpreter)?;
                    return Ok(b_true);
                } else if let Some(branch_false) = branch_false {
                    let b_false = branch_false.evaluate(interpreter)?;
                    return Ok(b_false);
                }
                Ok(JSValue::Undefined)
            }
            Stmt::Return(expr) => todo!(),
            Stmt::VariableDecl {
                is_mutable,
                identifier,
                initializer,
            } => {
                // establish the variable name
                let ident = identifier.evaluate(interpreter)?;
                if !ident.is_string() {
                    return Err(JSError::new("Expected string"));
                }
                let str_id = ident.to_string(interpreter)?;
                // right hand side is either the expr evaluation or undefined
                let rhs = match initializer {
                    Some(init_expr) => init_expr.evaluate(interpreter)?,
                    None => JSValue::Undefined,
                };
                // add a new variable to the variable heap
                interpreter.new_variable(str_id, *is_mutable, rhs);

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
