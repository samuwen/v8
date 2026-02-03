#![allow(dead_code)]
#![allow(unused_variables)]

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

                let ident = self.get_identifier()?;
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
                let ident = self.get_identifier()?;
                self.expect_and_consume(&Kind::LeftParen, "FunctionDecl")?;

                let parameters = if self.current_token.is_kind(&Kind::RightParen) {
                    vec![]
                } else {
                    let mut params = vec![];
                    let first_param = self.get_identifier()?;
                    params.push(first_param);
                    while self.current_token.is_kind(&Kind::Comma) {
                        self.next_token();
                        let param = self.get_identifier()?;
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

    // just to be consistent with the grammar
    fn handle_expressions(&mut self) -> JSResult<Expr> {
        self.handle_assignment()
    }

    fn handle_assignment(&mut self) -> JSResult<Expr> {
        let mut left = self.handle_call()?; // this either gets a call chain or an identifier
        if self.current_token.is_kinds(vec![
            Kind::Equals,
            Kind::PlusEquals,
            Kind::MinusEquals,
            Kind::StarEquals,
            Kind::SlashEquals,
        ]) {
            let op_token = self.current_token.clone();
            self.next_token();
            let peek = self._peek().ok_or(JSError::new("Unexpected EOF"))?;
            let right = if peek.is_kind(&Kind::Equals) {
                self.handle_assignment()?
            } else {
                self.handle_equality()?
            };
            if op_token.is_kind(&Kind::Equals) {
                // if normal do it normally
                left = Expr::new_assignment(left, right);
            } else {
                // otherwise unpack it
                let op = match op_token.get_kind() {
                    Kind::PlusEquals => Kind::Plus,
                    Kind::MinusEquals => Kind::Minus,
                    Kind::StarEquals => Kind::Star,
                    Kind::SlashEquals => Kind::Slash,
                    _ => panic!("add the kind to the if list, dork"),
                };
                let right = Expr::new_binary(
                    Token::new_from_span(op, &op_token.get_span()),
                    left.clone(),
                    right,
                );
                left = Expr::new_assignment(left, right);
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
        if self.current_token.is_unary_operator() {
            let operator = self.current_token.clone();
            self.next_token();
            let right = self.handle_unaries()?;
            return Ok(Expr::new_unary(operator, right));
        }
        Ok(self.handle_postfix()?)
    }

    fn handle_postfix(&mut self) -> JSResult<Expr> {
        let mut left = self.handle_call()?;
        if self.current_token.is_postfix() {
            let operator = self.current_token.clone();
            self.next_token();
            left = Expr::new_postfix(left, operator);
        }

        Ok(left)
    }

    fn handle_call(&mut self) -> JSResult<Expr> {
        let mut left = self.handle_primaries()?;
        while self
            .current_token
            .is_kinds(vec![Kind::Dot, Kind::LeftParen, Kind::LeftSquare])
        {
            let prev = self.current_token.clone();
            self.next_token();
            match prev.get_kind() {
                Kind::Dot => {
                    let ident = self.get_identifier()?;
                    left = Expr::new_object_call(ident);
                }
                Kind::LeftParen => {
                    println!("{left}");
                    let args = if self.current_token.is_kind(&Kind::RightParen) {
                        vec![]
                    } else {
                        let mut args = Vec::with_capacity(6);
                        let arg = self.handle_expressions()?;
                        args.push(arg);
                        while self.current_token.is_kind(&Kind::Comma) {
                            self.next_token();
                            let param = self.handle_expressions()?;
                            args.push(param);
                        }
                        args
                    };
                    self.expect_and_consume(&Kind::RightParen, "CallExpr")?;
                    left = Expr::new_function_call(left, args);
                }
                Kind::LeftSquare => {
                    let expr = self.handle_expressions()?;
                    self.expect_and_consume(&Kind::RightSquare, "CallExpr")?;
                    left = Expr::new_object_call(expr);
                }
                _ => (),
            }
        }
        Ok(left)
    }

    fn handle_primaries(&mut self) -> JSResult<Expr> {
        let current = self.current_token.clone();
        let current_span = self.current_token.get_span();
        let source_value = self
            .interpreter
            .get_source_at_span(&current_span)
            .to_string();
        self.next_token();
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
                self.next_token();
                // weird literal function expression syntax
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
                    let mut params = Vec::with_capacity(6); // that'd be a lotta args
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
                let created = JSValue::new_function(ident, parameters, body, self.interpreter)?;
                Ok(Expr::new_literal(created))
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

    fn get_identifier(&mut self) -> JSResult<Expr> {
        let current_span = self.current_token.get_span();
        let source_value = self
            .interpreter
            .get_source_at_span(&current_span)
            .to_string();
        check_identifier(&source_value)?;
        let idx = get_or_intern_string(&source_value);
        self.next_token();
        Ok(Expr::new_identifier(&idx))
    }
}
