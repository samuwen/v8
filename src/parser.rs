use std::{iter::Peekable, vec::IntoIter};

use crate::{
    Interpreter,
    errors::JSError,
    expr::Expr,
    global::get_or_intern_string,
    stmt::Stmt,
    token::{Kind, Token},
    utils::check_identifier,
    values::{ArrowFunctionReturn, JSResult, JSValue},
};

pub struct Parser<'a> {
    current_token: Token,
    errors: Vec<JSError>,
    had_error: bool,
    tokens: Peekable<IntoIter<Token>>,
    interpreter: &'a mut Interpreter,
}

impl<'a> Parser<'a> {
    pub fn new(token_list: Vec<Token>, interpreter: &'a mut Interpreter) -> Self {
        let mut iter = token_list.into_iter().peekable();
        let first_token = iter.next().unwrap_or(Token::new_eof());
        Self {
            current_token: first_token,
            errors: vec![],
            had_error: false,
            tokens: iter,
            interpreter,
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut program: Vec<Stmt> = vec![];
        while !self.current_token.is_kind(&Kind::Eof) {
            let stmt_res = self.handle_statements();
            match stmt_res {
                Ok(stmt) => {
                    program.push(stmt);
                }
                Err(e) => {
                    eprintln!("{}", e.message);
                    self.errors.push(e);
                    self.had_error = true;
                }
            }
        }
        program
    }

    fn handle_statements(&mut self) -> JSResult<Stmt> {
        match self.current_token.get_kind() {
            Kind::Let | Kind::Var | Kind::Const => {
                let is_mutable = self.current_token.is_kinds(vec![Kind::Let, Kind::Var]);
                self.next_token();

                let ident = self.handle_primaries()?; // get ident
                let expr = if self.current_token.is_kind(&Kind::Equals) {
                    self.next_token(); // consume equals
                    Some(self.handle_expressions()?)
                } else {
                    None
                };
                self.expect_and_consume(&Kind::Semicolon, "VariableDecl")?;
                Ok(Stmt::new_variable(is_mutable, ident, expr))
            }

            Kind::Function => {
                self.next_token();
                let ident = self.handle_expressions()?;
                self.expect_and_consume(&Kind::LeftParen, "FunctionDecl")?;

                let parameters = if self.current_token.is_kind(&Kind::RightParen) {
                    vec![]
                } else {
                    let mut params = vec![];
                    let first_param = self.handle_expressions()?;
                    params.push(first_param);
                    while self.current_token.is_kind(&Kind::Comma) {
                        self.next_token();
                        let param = self.handle_expressions()?;
                        params.push(param);
                    }
                    params
                };
                self.expect_and_consume(&Kind::RightParen, "FunctionDecl")?;
                let body = self.handle_statements()?;
                Ok(Stmt::new_function(ident, parameters, body))
            }

            Kind::LeftCurly => {
                self.next_token();
                let mut statements = vec![];
                while !self.current_token.is_kind(&Kind::RightCurly) {
                    let stmt = self.handle_statements()?;
                    statements.push(stmt);
                }
                self.expect_and_consume(&Kind::RightCurly, "BlockStatement")?;
                return Ok(Stmt::new_block(statements));
            }

            Kind::Return => {
                self.next_token();
                if self.current_token.is_kind(&Kind::Semicolon) {
                    self.expect_and_consume(&Kind::Semicolon, "ReturnStatement")?;
                    return Ok(Stmt::new_return(None));
                }
                let expr = self.handle_expressions()?;
                self.expect_and_consume(&Kind::Semicolon, "ReturnStatement")?;
                Ok(Stmt::new_return(Some(expr)))
            }

            Kind::Break => {
                self.next_token();
                self.expect_and_consume(&Kind::Semicolon, "BreakStatement")?;
                Ok(Stmt::Break)
            }

            Kind::Continue => {
                self.next_token();
                self.expect_and_consume(&Kind::Semicolon, "ContinueStatement")?;
                Ok(Stmt::Continue)
            }

            Kind::If => {
                self.next_token();
                self.expect_and_consume(&Kind::LeftParen, "IfStatement")?;
                let condition = self.handle_expressions()?;
                self.expect_and_consume(&Kind::RightParen, "IfStatement")?;
                let true_statement = self.handle_statements()?;
                let false_statement = if self.current_token.is_kind(&Kind::Else) {
                    self.next_token();
                    let false_statement = self.handle_statements()?;
                    Some(false_statement)
                } else {
                    None
                };
                Ok(Stmt::new_if(condition, true_statement, false_statement))
            }

            Kind::While => {
                self.next_token();
                self.expect_and_consume(&Kind::LeftParen, "WhileStatement")?;
                let expr = self.handle_expressions()?;
                self.expect_and_consume(&Kind::RightParen, "WhileStatement")?;
                let stmt = self.handle_statements()?;
                Ok(Stmt::new_while(expr, stmt))
            }

            Kind::For => {
                self.next_token();
                self.expect_and_consume(&Kind::LeftParen, "ForStatement")?;
                let initializer = if self.current_token.is_kind(&Kind::Semicolon) {
                    self.next_token(); // statements consume semis
                    None
                } else {
                    Some(self.handle_statements()?)
                };

                let condition = if self.current_token.is_kind(&Kind::Semicolon) {
                    None
                } else {
                    Some(self.handle_expressions()?)
                };
                self.expect_and_consume(&Kind::Semicolon, "ForStatement")?;

                let state = if self.current_token.is_kind(&Kind::RightParen) {
                    None
                } else {
                    Some(self.handle_expressions()?)
                };
                self.expect_and_consume(&Kind::RightParen, "ForStatement")?;

                let body = self.handle_statements()?;

                Ok(Stmt::new_for(initializer, condition, state, body))
            }

            _ => {
                let expr = self.handle_expressions()?;
                self.expect_and_consume(&Kind::Semicolon, "ExpressionStatement")?;
                Ok(Stmt::new_expression(expr))
            }
        }
    }

    fn handle_expressions(&mut self) -> JSResult<Expr> {
        let mut left = self.handle_equality()?;
        if self
            .current_token
            .is_kinds(vec![Kind::Equals, Kind::PlusEquals])
        {
            let op_token = self.current_token.clone();
            self.next_token();
            let right = self.handle_expressions()?;
            match op_token.get_kind() {
                Kind::PlusEquals => {
                    let right = Expr::new_binary(
                        Token::new_from_span(Kind::Plus, &op_token.get_span()),
                        left.clone(),
                        right,
                    );
                    left = Expr::new_assignment(left, right);
                }
                Kind::Plus => {
                    left = Expr::new_assignment(left, right);
                }
                _ => unimplemented!(),
            }
        }
        Ok(left)
    }

    fn handle_equality(&mut self) -> JSResult<Expr> {
        let mut left = self.handle_comparisons()?;
        while self
            .current_token
            .is_kinds(vec![Kind::EqualEqual, Kind::NotEqual])
        {
            let operator = self.current_token.clone();
            self.next_token();
            let right = self.handle_comparisons()?;
            left = Expr::new_binary(operator, left, right);
        }
        Ok(left)
    }

    fn handle_comparisons(&mut self) -> JSResult<Expr> {
        let mut left = self.handle_terms()?;
        while self.current_token.is_kinds(vec![
            Kind::LessThan,
            Kind::GreaterThan,
            Kind::LessThanOrEquals,
            Kind::GreaterThanOrEquals,
        ]) {
            let operator = self.current_token.clone();
            self.next_token();
            let right = self.handle_terms()?;
            left = Expr::new_binary(operator, left, right);
        }
        Ok(left)
    }

    fn handle_terms(&mut self) -> JSResult<Expr> {
        let mut left = self.handle_factors()?;
        while self.current_token.is_kinds(vec![Kind::Plus, Kind::Minus]) {
            let operator = self.current_token.clone();
            self.next_token();
            let right = self.handle_factors()?;
            left = Expr::new_binary(operator, left, right);
        }
        Ok(left)
    }

    fn handle_factors(&mut self) -> JSResult<Expr> {
        let mut left = self.handle_unaries()?;
        while self
            .current_token
            .is_kinds(vec![Kind::Star, Kind::Slash, Kind::Percent])
        {
            let operator = self.current_token.clone();
            self.next_token();
            let right = self.handle_unaries()?;
            left = Expr::new_binary(operator, left, right);
        }
        Ok(left)
    }

    fn handle_unaries(&mut self) -> JSResult<Expr> {
        if self.current_token.is_kinds(vec![
            Kind::Minus,
            Kind::Bang,
            Kind::Typeof,
            Kind::Plus,
            Kind::Void,
        ]) {
            let operator = self.current_token.clone();
            self.next_token();
            let right = self.handle_unaries()?;
            return Ok(Expr::new_unary(operator, right));
        }
        let expr = self.handle_primaries()?;
        Ok(expr)
    }

    fn handle_primaries(&mut self) -> JSResult<Expr> {
        let current = self.current_token.clone();
        let current_span = current.get_span();
        self.next_token();
        let source_value = self
            .interpreter
            .get_source_at_span(&current_span)
            .to_string();
        match current.get_kind() {
            Kind::Number => {
                let num = source_value
                    .parse::<f64>()
                    .map_err(|_| JSError::new("Invalid number"))?;
                return Ok(Expr::new_literal(JSValue::new_number(&num)));
            }
            Kind::String => {
                let idx = get_or_intern_string(&source_value);
                Ok(Expr::new_literal(JSValue::new_string(&idx)))
            }
            Kind::Identifier => {
                check_identifier(&source_value)?;
                let idx = get_or_intern_string(&source_value);
                Ok(Expr::new_identifier(&idx))
            }
            Kind::True => Ok(Expr::new_literal(JSValue::new_boolean(&true))),
            Kind::False => Ok(Expr::new_literal(JSValue::new_boolean(&false))),
            Kind::Null => Ok(Expr::new_literal(JSValue::new_null())),
            Kind::Undefined => Ok(Expr::new_literal(JSValue::new_undefined())),
            Kind::LeftParen => {
                // immediate right paren - we're in arrow land. grouping needs inner content
                if self.current_token.is_kind(&Kind::RightParen) {
                    self.next_token();
                    self.expect_and_consume(&Kind::Arrow, "ArrowFunction")?;
                    let raw_body = self.handle_statements()?;
                    let body = if let Stmt::Expression(expr) = raw_body {
                        ArrowFunctionReturn::Expr(expr)
                    } else {
                        ArrowFunctionReturn::Stmt(Box::new(raw_body))
                    };

                    return Ok(Expr::new_literal(JSValue::new_arrow_function(
                        vec![],
                        body,
                        self.interpreter,
                    )));
                }

                let expr = self.handle_expressions()?;
                // comma separator means arrow land
                if self.current_token.is_kind(&Kind::Comma) {
                    let mut args = Vec::with_capacity(8);
                    args.push(expr);
                    while self.current_token.is_kind(&Kind::Comma) {
                        self.next_token();
                        let param = self.handle_expressions()?;
                        args.push(param);
                    }
                    self.expect_and_consume(&Kind::RightParen, "ArrowFunction")?;
                    self.expect_and_consume(&Kind::Arrow, "ArrowFunction")?;

                    let body = if self.current_token.is_kind(&Kind::LeftCurly) {
                        let stmt = self.handle_statements()?;
                        ArrowFunctionReturn::Stmt(Box::new(stmt))
                    } else {
                        let expr = self.handle_expressions()?;
                        ArrowFunctionReturn::Expr(Box::new(expr))
                    };

                    Ok(Expr::new_literal(JSValue::new_arrow_function(
                        args,
                        body,
                        self.interpreter,
                    )))
                } else {
                    self.expect_and_consume(&Kind::RightParen, "Expression")?;
                    // if next token is an arrow we're in arrow land
                    if self.current_token.is_kind(&Kind::Arrow) {
                        let raw_body = self.handle_statements()?;
                        let body = if let Stmt::Expression(expr) = raw_body {
                            ArrowFunctionReturn::Expr(expr)
                        } else {
                            ArrowFunctionReturn::Stmt(Box::new(raw_body))
                        };

                        return Ok(Expr::new_literal(JSValue::new_arrow_function(
                            vec![expr; 1],
                            body,
                            self.interpreter,
                        )));
                    }
                    // otherwise its just a parenthetical
                    Ok(Expr::new_grouping(expr))
                }
            }
            Kind::LeftSquare => {
                if self.current_token.is_kind(&Kind::RightSquare) {
                    self.next_token();
                    return Ok(Expr::new_literal(JSValue::new_array(vec![])));
                }
                // js ecosystem typically frowns on this many arguments
                let mut expressions = Vec::with_capacity(6);
                let expr = self.handle_expressions()?;
                expressions.push(expr);
                while self.current_token.is_kind(&Kind::Comma) {
                    self.next_token();
                    let expr = self.handle_expressions()?;
                    expressions.push(expr);
                }

                self.expect_and_consume(&Kind::RightSquare, "ArrayExpression")?;
                Ok(Expr::new_literal(JSValue::new_array(expressions)))
            }
            Kind::LeftCurly => {
                if self.current_token.is_kind(&Kind::RightCurly) {
                    self.next_token();
                    return Ok(Expr::new_literal(JSValue::new_object(
                        vec![],
                        self.interpreter,
                    )));
                }

                let mut properties = Vec::with_capacity(8);

                let key_error = JSError::new("Object literal key must be a string");
                loop {
                    let key = match self.current_token.get_kind() {
                        Kind::Identifier | Kind::String => self
                            .interpreter
                            .get_source_at_span(&self.current_token.get_span()),
                        _ => return Err(key_error),
                    };
                    self.next_token();
                    let key_index = get_or_intern_string(&key);
                    self.expect_and_consume(&Kind::Colon, "ObjectExpression")?;
                    let value_expr = self.handle_expressions()?;
                    let value = value_expr.evaluate(self.interpreter)?;
                    properties.push((key_index, value));

                    if !self.current_token.is_kind(&Kind::Comma) {
                        break;
                    }
                    self.next_token();
                }

                self.expect_and_consume(&Kind::RightCurly, "ObjectExpression")?;
                return Ok(Expr::new_literal(JSValue::new_object(
                    properties,
                    self.interpreter,
                )));
            }
            Kind::Function => {
                // weird literal function expression syntax
                self.next_token();
                let ident = if self.current_token.is_kind(&Kind::LeftParen) {
                    None
                } else {
                    let expr = self.handle_expressions()?;
                    Some(expr)
                };
                self.expect_and_consume(&Kind::LeftParen, "FunctionExpression")?;

                let parameters = if self.current_token.is_kind(&Kind::RightParen) {
                    vec![]
                } else {
                    let mut params = vec![];
                    let first_param = self.handle_expressions()?;
                    params.push(first_param);
                    while self.current_token.is_kind(&Kind::Comma) {
                        self.next_token();
                        let param = self.handle_expressions()?;
                        params.push(param);
                    }
                    params
                };
                self.expect_and_consume(&Kind::RightParen, "FunctionExpression")?;
                let body = self.handle_statements()?;
                Ok(Expr::new_literal(JSValue::new_function(
                    ident, parameters, body,
                )))
            }
            token => Err(JSError::new(&format!("Unexpected token: {:?}", token))),
        }
    }

    fn next_token(&mut self) {
        if let Some(tok) = self.tokens.next() {
            self.current_token = tok;
        }
    }

    fn _peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn expect_and_consume(&mut self, kind: &Kind, caller: &str) -> JSResult<bool> {
        if self.current_token.is_kind(kind) {
            self.next_token();
            return Ok(true);
        }
        let error = JSError::new(&format!("Expected '{:?}' after {}", kind, caller));
        Err(error)
    }
}
