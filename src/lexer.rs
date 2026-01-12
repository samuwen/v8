use std::{iter::Peekable, str::Chars};

use crate::{
    global::SharedContext,
    token::{Kind, Token},
};

#[derive(Debug)]
pub struct Lexer<'a, 'b> {
    context: &'b mut SharedContext,
    current_char: char,
    current_column: usize,
    errors: Vec<LexerError>,
    had_error: bool,
    line: usize,
    start: usize,
    source: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
}

impl<'a, 'b> Lexer<'a, 'b> {
    pub fn new(context: &'b mut SharedContext, source: &'a str) -> Self {
        let chars = source.chars().peekable();
        Self {
            context,
            current_column: 0,
            current_char: '\0', // should never be read, so initialize to EOF
            errors: vec![],
            had_error: false,
            line: 1,
            start: 0,
            source: chars,
            tokens: Vec::with_capacity(100),
        }
    }

    pub fn lex(&mut self) {
        loop {
            self.next_char();
            if self.current_char == '\0' {
                break;
            }

            match self.current_char.to_ascii_lowercase() {
                '0'..='9' => {
                    self.start = self.current_column;
                    let mut digit_value = String::from(self.current_char);
                    while let Some(&v) = self.peek_next_char() {
                        match v {
                            '0'..='9' => {
                                let next = self.next_char();
                                digit_value.push(next);
                            }
                            '.' => {
                                self.next_char(); // throw it away
                                digit_value.push('.');
                            }
                            '_' => {
                                self.next_char(); // throw it away
                            }
                            _ => {
                                // break - we're done
                                break;
                            }
                        }
                    }
                    if !self.had_error {
                        let number = digit_value.parse::<f64>();
                        match number {
                            Ok(num) => self.add_token(Kind::Number(num)),
                            Err(e) => {
                                eprintln!("{e}");
                                self.report_error("f64 failed to parse");
                            }
                        }
                    }
                }

                // identifier
                'a'..='z' => {
                    self.start = self.current_column;
                    let mut ident = String::new();
                    while !self.is_whitespace() {
                        match self.current_char.to_ascii_lowercase() {
                            'a'..'z' | '_' | '0'..'9' => {
                                ident.push(self.current_char);
                            }
                            _ => {
                                let message = format!(
                                    "Invalid identifier character: '{}'",
                                    self.current_char
                                );
                                self.report_error(&message);
                            }
                        }
                        let peeked = self.peek_next_char();
                        if let Some(c) = peeked {
                            if c.is_whitespace() {
                                break;
                            }
                        }
                        self.next_char();
                    }

                    match ident.as_str() {
                        "let" => {
                            self.add_token(Kind::Let);
                        }
                        "const" => {
                            self.add_token(Kind::Const);
                        }
                        "function" => {
                            self.add_token(Kind::Function);
                        }
                        "return" => {
                            self.add_token(Kind::Return);
                        }
                        "if" => {
                            self.add_token(Kind::If);
                        }
                        "else" => {
                            self.add_token(Kind::Else);
                        }
                        "var" => {
                            self.add_token(Kind::Var);
                        }
                        _ => {
                            let idx = self.context.string_pool.add_string(&ident);
                            self.add_token(Kind::Identifier(idx))
                        }
                    }
                }

                '"' => {
                    self.lex_string('"');
                }
                '\'' => {
                    self.lex_string('\'');
                }
                '\n' => {
                    self.line += 1;
                }
                ' ' | '\t' => {
                    self.current_column += 1;
                }
                '+' => {
                    self.add_token(Kind::Plus);
                }
                '-' => {
                    self.add_token(Kind::Minus);
                }
                '*' => {
                    self.add_token(Kind::Star);
                }
                '/' => {
                    let is_single_comment = self.check_peeked_char('/');
                    if is_single_comment {
                        self.next_char(); // consume the other slash
                        while self.current_char != '\n' {
                            self.next_char();
                        }
                        self.line += 1;
                    } else {
                        self.add_token(Kind::Slash);
                    }
                }
                '=' => {
                    let is_double_equal = self.check_peeked_char('=');
                    if is_double_equal {
                        self.next_char();
                        let is_triple_equal = self.check_peeked_char('=');
                        if is_triple_equal {
                            self.next_char();
                            self.add_token(Kind::EqualEqualEqual);
                        } else {
                            self.add_token(Kind::EqualEqual);
                        }
                    } else {
                        self.add_token(Kind::Equals);
                    }
                }
                '(' => {
                    self.add_token(Kind::LeftParen);
                }
                ')' => {
                    self.add_token(Kind::RightParen);
                }
                '{' => {
                    self.add_token(Kind::LeftBrace);
                }
                '}' => {
                    self.add_token(Kind::RightBrace);
                }
                ';' => {
                    self.add_token(Kind::Semicolon);
                }
                ',' => {
                    self.add_token(Kind::Comma);
                }
                '\0' => {
                    self.add_token(Kind::Eof);
                }
                _ => {
                    let message = format!("Unhandled character: '{}'", self.current_char);
                    self.report_error(&message);
                }
            }
        }
    }

    pub fn had_errors(&mut self) -> bool {
        self.errors.len() > 0
    }

    pub fn replay_errors(&mut self) {
        for error in &self.errors {
            println!("{error}");
        }
    }

    pub fn print_tokens(&self) {
        for token in &self.tokens {
            token.print(&self.context.string_pool);
        }
    }

    fn next_char(&mut self) -> char {
        self.current_column += 1;
        self.current_char = self.source.next().unwrap_or('\0');
        self.current_char
    }

    fn peek_next_char(&mut self) -> Option<&char> {
        self.source.peek()
    }

    fn report_error(&mut self, message: &str) {
        let error = LexerError::new(message, self.line, self.start);
        self.errors.push(error);
        self.had_error = true;
        self.find_next_gap();
    }

    /// Try to reset the lexer state to the next whitespace
    fn find_next_gap(&mut self) {
        loop {
            let c = self.next_char();
            if c.is_whitespace() || c == '\0' {
                return;
            }
        }
    }

    fn add_token(&mut self, kind: Kind) {
        self.tokens
            .push(Token::new(kind, self.line, self.start, self.current_column))
    }

    fn lex_string(&mut self, terminator: char) {
        self.start = self.current_column;
        let mut string = String::new();
        self.next_char(); // discard the quote
        while self.current_char != terminator {
            let error_message = format!("Improperly terminated string: {}", string);
            if self.current_char == '\0' {
                self.report_error(&error_message);
                return;
            }
            string.push(self.current_char);

            if self.current_char == '\\' {
                let maybe_peek = self.peek_next_char();
                if let Some(c) = maybe_peek {
                    if *c == '\n' {
                        self.report_error(&error_message);
                        break;
                    }
                }
            }
            self.current_char = self.next_char();
        }
        if !self.had_error {
            let idx = self.context.string_pool.add_string(&string);
            self.add_token(Kind::String(idx));
        }
    }

    fn is_whitespace(&self) -> bool {
        self.current_char == '\0' || self.current_char.is_whitespace()
    }

    fn check_peeked_char(&mut self, check_char: char) -> bool {
        let peeked = self.peek_next_char();
        if let Some(&ch) = peeked {
            return ch == check_char;
        }
        false
    }
}

#[derive(Debug, Clone)]
pub struct LexerError {
    column: usize,
    line: usize,
    message: String,
}

impl LexerError {
    pub fn new(message: &str, line: usize, column: usize) -> Self {
        Self {
            message: message.to_owned(),
            line,
            column,
        }
    }
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = format!(
            "[ERROR:LEXER]: {} at line {}, column: {}",
            self.message, self.line, self.column
        );
        write!(f, "{}", message)
    }
}
