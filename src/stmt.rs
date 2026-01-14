use std::fmt;

use crate::expr::Expr;

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
