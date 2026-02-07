use crate::token::{Kind, Token, get_keyword};
use std::{iter::Peekable, str::Chars};

#[derive(Debug)]
pub struct Lexer<'a> {
    current_char: char,
    current_column: usize,
    errors: Vec<LexerError>,
    had_error: bool,
    line: usize,
    start: usize,
    source: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.chars().peekable();
        let first_char = chars.next().unwrap_or('\0');
        Self {
            current_column: 0,
            current_char: first_char.clone(),
            errors: vec![],
            had_error: false,
            line: 1,
            start: 0,
            source: chars,
            tokens: Vec::with_capacity(100),
        }
    }

    pub fn lex(&mut self) -> Vec<Token> {
        loop {
            if self.current_char == '\0' {
                self.add_token(Kind::Eof);
                break;
            }

            self.start = self.current_column;
            match self.current_char.to_ascii_lowercase() {
                '0'..='9' => {
                    loop {
                        match self.current_char.to_ascii_lowercase() {
                            '0'..='9' | '.' => {
                                self.next_char();
                            }
                            '_' => {
                                self.next_char(); // discard
                            }
                            _ => break,
                        }
                    }
                    self.add_token(Kind::Number);
                }

                // identifier
                'a'..='z' => {
                    let mut ident = String::new();
                    loop {
                        match self.current_char.to_ascii_lowercase() {
                            'a'..='z' | '_' | '0'..'9' => {
                                ident.push(self.current_char);
                                self.next_char();
                            }
                            _ => break,
                        }
                    }

                    let maybe_keyword = get_keyword(&ident.to_lowercase());
                    if let Some(kind) = maybe_keyword {
                        self.add_token(kind);
                    } else {
                        self.add_token(Kind::Identifier);
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
                    self.next_char();
                }
                ' ' | '\t' => {
                    self.next_char();
                }
                '.' => {
                    self.add_token_and_advance(Kind::Dot);
                }
                '+' => {
                    if self.check_peeked_char('+') {
                        self.next_char();
                        self.add_token_and_advance(Kind::PlusPlus);
                    } else {
                        if self.check_peeked_char('=') {
                            self.next_char();
                            self.add_token_and_advance(Kind::PlusEquals);
                        } else {
                            self.add_token_and_advance(Kind::Plus);
                        }
                    }
                }
                '-' => {
                    let is_second_minus = self.check_peeked_char('-');
                    if is_second_minus {
                        self.next_char();
                        self.add_token_and_advance(Kind::MinusMinus);
                    } else {
                        if self.check_peeked_char('=') {
                            self.next_char();
                            self.add_token_and_advance(Kind::MinusEquals);
                        } else {
                            self.add_token_and_advance(Kind::Minus);
                        }
                    }
                }
                '*' => {
                    let is_equals = self.check_peeked_char('=');
                    if is_equals {
                        self.next_char();
                        self.add_token_and_advance(Kind::StarEquals);
                    } else {
                        self.add_token_and_advance(Kind::Star);
                    }
                }
                '/' => {
                    let is_single_comment = self.check_peeked_char('/');
                    if is_single_comment {
                        while self.current_char != '\n' {
                            self.next_char();
                        }
                        self.line += 1;
                        self.next_char();
                    } else {
                        let is_equals = self.check_peeked_char('=');
                        if is_equals {
                            self.next_char();
                            self.add_token_and_advance(Kind::SlashEquals);
                        } else {
                            self.add_token_and_advance(Kind::Slash);
                        }
                    }
                }
                '=' => {
                    let is_double_equal = self.check_peeked_char('=');
                    if is_double_equal {
                        self.next_char();
                        let is_triple_equal = self.check_peeked_char('=');
                        if is_triple_equal {
                            self.next_char();
                            self.add_token_and_advance(Kind::EqualEqualEqual);
                        } else {
                            self.add_token_and_advance(Kind::EqualEqual);
                        }
                    } else {
                        let is_arrow = self.check_peeked_char('>');
                        if is_arrow {
                            self.next_char();
                            self.add_token_and_advance(Kind::Arrow);
                        } else {
                            self.add_token_and_advance(Kind::Equals);
                        }
                    }
                }
                ':' => {
                    self.add_token_and_advance(Kind::Colon);
                }
                '(' => {
                    self.add_token_and_advance(Kind::LeftParen);
                }
                ')' => {
                    self.add_token_and_advance(Kind::RightParen);
                }
                '{' => {
                    self.add_token_and_advance(Kind::LeftCurly);
                }
                '}' => {
                    self.add_token_and_advance(Kind::RightCurly);
                }
                ';' => {
                    self.add_token_and_advance(Kind::Semicolon);
                }
                ',' => {
                    self.add_token_and_advance(Kind::Comma);
                }
                '!' => {
                    let is_not_equals = self.check_peeked_char('=');
                    if is_not_equals {
                        self.next_char();
                        self.add_token_and_advance(Kind::NotEqual);
                    } else {
                        self.add_token_and_advance(Kind::Bang);
                    }
                }
                '<' => {
                    let is_equals = self.check_peeked_char('=');
                    if is_equals {
                        self.next_char();
                        self.add_token_and_advance(Kind::LessThanOrEquals);
                    } else {
                        self.add_token_and_advance(Kind::LessThan);
                    }
                }
                '>' => {
                    let is_equals = self.check_peeked_char('=');
                    if is_equals {
                        self.next_char();
                        self.add_token_and_advance(Kind::GreaterThanOrEquals);
                    } else {
                        self.add_token_and_advance(Kind::GreaterThan);
                    }
                }
                '[' => {
                    self.add_token_and_advance(Kind::LeftSquare);
                }
                ']' => {
                    self.add_token_and_advance(Kind::RightSquare);
                }
                '%' => {
                    self.add_token_and_advance(Kind::Percent);
                }
                '\0' => {
                    self.add_token_and_advance(Kind::Eof);
                }
                _ => {
                    let message = format!("Unhandled character: '{}'", self.current_char);
                    self.report_error(&message);
                }
            }
        }
        self.tokens.clone()
    }

    pub fn had_errors(&mut self) -> bool {
        self.errors.len() > 0
    }

    pub fn replay_errors(&mut self) {
        for error in &self.errors {
            eprintln!("{error}");
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

    fn add_token_and_advance(&mut self, kind: Kind) {
        self.next_char();
        self.add_token(kind);
    }

    fn lex_string(&mut self, terminator: char) {
        self.next_char(); // discard the quote
        self.start = self.current_column;
        let mut string = String::new();
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
            self.next_char();
        }
        if !self.had_error {
            self.add_token(Kind::String);
            self.next_char();
        }
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
