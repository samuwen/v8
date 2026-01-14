use std::vec::IntoIter;

use crate::{
    expr::Expr,
    stmt::Stmt,
    token::{Kind, Token},
    value::{ArrowFunctionReturn, Value},
};

pub struct Parser {
    current_token: Token,
    errors: Vec<ParserError>,
    tokens: IntoIter<Token>,
}

impl Parser {
    pub fn new(token_list: Vec<Token>) -> Self {
        let mut iter = token_list.into_iter();
        let first_token = iter.next().unwrap_or(Token::new_eof());
        Self {
            current_token: first_token,
            errors: vec![],
            tokens: iter,
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
                    break;
                }
            }
        }
        program
    }

    fn handle_statements(&mut self) -> Result<Stmt, ParserError> {
        match self.current_token.get_kind() {
            Kind::Let | Kind::Var | Kind::Const => {
                let is_mutable = self.current_token.is_kinds(vec![Kind::Let, Kind::Var]);
                self.next_token();

                let ident = self.handle_expressions()?;
                let expr = if self.current_token.is_kind(&Kind::Equals) {
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
                let mut statments = vec![];
                while !self.current_token.is_kind(&Kind::RightCurly) {
                    let stmt = self.handle_statements()?;
                    statments.push(stmt);
                }
                self.expect_and_consume(&Kind::RightCurly, "BlockStatement")?;
                Ok(Stmt::new_block(statments))
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
                    None
                } else {
                    Some(self.handle_statements()?)
                };
                self.expect_and_consume(&Kind::Semicolon, "ForStatement")?;

                let condition = if self.current_token.is_kind(&Kind::Semicolon) {
                    None
                } else {
                    Some(self.handle_expressions()?)
                };
                self.expect_and_consume(&Kind::Semicolon, "ForStatement")?;

                let state = if self.current_token.is_kind(&Kind::Semicolon) {
                    self.next_token();
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

    fn handle_expressions(&mut self) -> Result<Expr, ParserError> {
        let left = self.handle_equality()?;
        while self.current_token.is_kind(&Kind::Equals) {
            self.next_token();
            let right = self.handle_expressions()?;
            if let Expr::Variable(_) = left {
                return Ok(Expr::new_assignment(left, right));
            }
            let error = ParserError::new("Left side of assignment is not an identifier");
            return Err(error);
        }
        Ok(left)
    }

    fn handle_equality(&mut self) -> Result<Expr, ParserError> {
        let left = self.handle_comparisons()?;
        while self
            .current_token
            .is_kinds(vec![Kind::EqualEqual, Kind::NotEqual])
        {
            let operator = self.current_token.clone();
            self.next_token();
            let right = self.handle_comparisons()?;
            return Ok(Expr::new_binary(operator, left, right));
        }
        Ok(left)
    }

    fn handle_comparisons(&mut self) -> Result<Expr, ParserError> {
        let left = self.handle_terms()?;
        while self
            .current_token
            .is_kinds(vec![Kind::LessThan, Kind::GreaterThan])
        {
            let operator = self.current_token.clone();
            self.next_token();
            let right = self.handle_terms()?;
            return Ok(Expr::new_binary(operator, left, right));
        }
        Ok(left)
    }

    fn handle_terms(&mut self) -> Result<Expr, ParserError> {
        let left = self.handle_factors()?;
        while self.current_token.is_kinds(vec![Kind::Plus, Kind::Minus]) {
            let operator = self.current_token.clone();
            self.next_token();
            let right = self.handle_factors()?;
            return Ok(Expr::new_binary(operator, left, right));
        }
        Ok(left)
    }

    fn handle_factors(&mut self) -> Result<Expr, ParserError> {
        let left = self.handle_unaries()?;
        while self.current_token.is_kinds(vec![Kind::Star, Kind::Slash]) {
            let operator = self.current_token.clone();
            self.next_token();
            let right = self.handle_unaries()?;
            return Ok(Expr::new_binary(operator, left, right));
        }
        Ok(left)
    }

    fn handle_unaries(&mut self) -> Result<Expr, ParserError> {
        while self.current_token.is_kinds(vec![Kind::Minus, Kind::Bang]) {
            let operator = self.current_token.clone();
            self.next_token();
            let right = self.handle_unaries()?;
            return Ok(Expr::new_unary(operator, right));
        }
        self.handle_primaries()
    }

    fn handle_primaries(&mut self) -> Result<Expr, ParserError> {
        let current = self.current_token.clone();
        self.next_token();
        match current.get_kind() {
            Kind::Number(num) => Ok(Expr::new_literal(Value::new_number(num))),
            Kind::String(idx) => Ok(Expr::new_literal(Value::new_string(idx))),
            Kind::Identifier(idx) => Ok(Expr::new_variable(idx)),
            Kind::True => Ok(Expr::new_literal(Value::new_boolean(&true))),
            Kind::False => Ok(Expr::new_literal(Value::new_boolean(&false))),
            Kind::Null => Ok(Expr::new_literal(Value::new_null())),
            Kind::Undefined => Ok(Expr::new_literal(Value::new_undefined())),
            Kind::LeftParen => {
                // immediate right paren - we're in arrow land. grouping needs guts
                if self.current_token.is_kind(&Kind::RightParen) {
                    self.next_token();
                    self.expect_and_consume(&Kind::Arrow, "ArrowFunction")?;
                    let raw_body = self.handle_statements()?;
                    let body = if let Stmt::Expression(expr) = raw_body {
                        ArrowFunctionReturn::Expr(expr)
                    } else {
                        ArrowFunctionReturn::Stmt(Box::new(raw_body))
                    };

                    return Ok(Expr::new_literal(Value::new_arrow_function(vec![], body)));
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

                    Ok(Expr::new_literal(Value::new_arrow_function(args, body)))
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

                        return Ok(Expr::new_literal(Value::new_arrow_function(
                            vec![expr; 1],
                            body,
                        )));
                    }
                    // otherwise its just a parenthetical
                    Ok(Expr::new_grouping(expr))
                }
            }
            Kind::LeftSquare => {
                if self.current_token.is_kind(&Kind::RightSquare) {
                    self.next_token();
                    return Ok(Expr::new_literal(Value::new_array(vec![])));
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
                Ok(Expr::new_literal(Value::new_array(expressions)))
            }
            Kind::LeftCurly => {
                if self.current_token.is_kind(&Kind::RightCurly) {
                    self.next_token();
                    return Ok(Expr::new_literal(Value::new_object(vec![])));
                }

                let mut pairs = Vec::with_capacity(8);

                loop {
                    let key = self.handle_primaries()?;
                    let key_index = if let Expr::Literal(Value::String(idx)) = key {
                        idx
                    } else if let Expr::Variable(idx) = key {
                        idx
                    } else {
                        return Err(ParserError::new("Object literal key must be a string"));
                    };
                    self.expect_and_consume(&Kind::Colon, "ObjectExpression")?;
                    let value = self.handle_expressions()?;
                    pairs.push((key_index, value));

                    if !self.current_token.is_kind(&Kind::Comma) {
                        break;
                    }
                    self.next_token();
                }

                self.expect_and_consume(&Kind::RightCurly, "ObjectExpression")?;
                return Ok(Expr::new_literal(Value::new_object(pairs)));
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
                Ok(Expr::new_literal(Value::new_function(
                    ident, parameters, body,
                )))
            }
            token => Err(ParserError::new(&format!("Unexpected token: {:?}", token))),
        }
    }

    fn next_token(&mut self) {
        if let Some(tok) = self.tokens.next() {
            self.current_token = tok;
        }
    }

    fn expect_and_consume(&mut self, kind: &Kind, caller: &str) -> Result<bool, ParserError> {
        if self.current_token.is_kind(kind) {
            self.next_token();
            return Ok(true);
        }
        let error = ParserError::new(&format!("Expected '{:?}' after {}", kind, caller));
        Err(error)
    }
}

#[derive(Debug, Clone)]
pub struct ParserError {
    message: String,
}

impl ParserError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_owned(),
        }
    }
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = format!("[ERROR:PARSER]: {}", self.message);
        write!(f, "{}", message)
    }
}
