#[derive(Debug, PartialEq, Clone)]
pub enum Kind {
    // Literals
    Number(f64),
    String(usize),     // index to string map
    Identifier(usize), // index to string map

    // Keywords (start with just these)
    Let,
    Const,
    Function,
    Return,
    If,
    Else,

    // Operators (minimal set)
    Plus,
    Minus,
    Star,
    Slash,
    Equals,
    EqualEqual,

    // Punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    Comma,

    Eof,
}

#[derive(Debug)]
pub struct Token {
    kind: Kind,
    line: usize,
    start: usize,
    end: usize,
}

impl Token {
    pub fn new(kind: Kind, line: usize, start: usize, end: usize) -> Self {
        Self {
            kind,
            line,
            start,
            end,
        }
    }

    pub fn get_kind(&self) -> &Kind {
        &self.kind
    }

    pub fn to_string(&self, kind_str: Option<String>) -> String {
        if let Some(string) = kind_str {
            return format!(
                "{} | line: {} | start: {} | end: {}",
                string, self.line, self.start, self.end
            );
        }
        return format!(
            "{:?} | line: {} | start: {} | end: {}",
            self.kind, self.line, self.start, self.end
        );
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} | line: {} | start: {} | end: {}",
            self.kind, self.line, self.start, self.end
        )
    }
}
