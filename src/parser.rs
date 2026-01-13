use std::vec::IntoIter;

use crate::{
    expr::{Expr, Value},
    stmt::Stmt,
    token::{Kind, Token},
};

pub struct Parser {
    current_token: Token,
    errors: Vec<ParserError>,
    tokens: IntoIter<Token>,
}

impl Parser {
    pub fn new(token_list: Vec<Token>) -> Self {
        if token_list.len() == 0 {
            eprintln!("Empty token list passed to parser. Wiring logic wrong?");
        }
        let mut iter = token_list.into_iter();
        let first_token = iter.next().unwrap(); // safe because we know we have at least one token
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

            Kind::LeftBrace => {
                self.next_token();
                let mut statments = vec![];
                while !self.current_token.is_kind(&Kind::RightBrace) {
                    let stmt = self.handle_statements()?;
                    statments.push(stmt);
                }
                self.expect_and_consume(&Kind::RightBrace, "BlockStatement")?;
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
        match current.get_kind() {
            Kind::Number(num) => {
                self.next_token();
                Ok(Expr::new_literal(Value::new_number(num)))
            }
            Kind::String(idx) => {
                self.next_token();
                Ok(Expr::new_literal(Value::new_string(idx)))
            }
            Kind::Identifier(idx) => {
                self.next_token();
                Ok(Expr::new_variable(idx))
            }
            Kind::True => {
                self.next_token();
                Ok(Expr::new_literal(Value::new_boolean(&true)))
            }
            Kind::False => {
                self.next_token();
                Ok(Expr::new_literal(Value::new_boolean(&false)))
            }
            Kind::LeftParen => {
                self.next_token(); // consume '('
                let expr = self.handle_expressions()?;
                self.expect_and_consume(&Kind::RightParen, "Expression")?;
                Ok(Expr::new_grouping(expr))
            }
            token => Err(ParserError::new(&format!("Unexpected token: {:?}", token))),
        }
    }

    fn next_token(&mut self) {
        if let Some(tok) = self.tokens.next() {
            self.current_token = tok;
            println!(
                "Advancing to next token. Now: {:?}",
                self.current_token.get_kind()
            );
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
