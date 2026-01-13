use crate::string_pool::StringPool;

#[derive(Debug, PartialEq, Clone)]
pub enum Kind {
    // Literals
    Number(f64),
    String(usize),     // index to string map
    Identifier(usize), // index to string map
    True,
    False,

    // Keywords (start with just these)
    Var,
    Let,
    Const,
    Function,
    Return,
    If,
    Else,
    Break,
    Continue,
    While,

    // Operators (minimal set)
    Plus,
    Minus,
    Star,
    Slash,
    Bang,
    Equals,
    EqualEqual,
    EqualEqualEqual,
    NotEqual,
    LessThan,
    GreaterThan,

    // Punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    Comma,

    Eof,
}

#[derive(Clone, Debug)]
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

    pub fn new_eof() -> Self {
        Self {
            kind: Kind::Eof,
            line: 0,
            start: 0,
            end: 0,
        }
    }

    pub fn print(&self, pool: &StringPool) {
        match self.kind {
            Kind::Identifier(idx) | Kind::String(idx) => {
                let as_str = pool.get_string_by_idx(idx);
                if let Some(string) = as_str {
                    println!(
                        "Identifier: '{}' | line: {} | start: {} | end: {}",
                        string, self.line, self.start, self.end
                    );
                }
            }
            _ => {
                println!(
                    "{:?} | line: {} | start: {} | end: {}",
                    self.kind, self.line, self.start, self.end
                );
            }
        }
    }

    pub fn is_kind(&self, kind: &Kind) -> bool {
        &self.kind == kind
    }

    pub fn is_kinds(&self, kinds: Vec<Kind>) -> bool {
        kinds.iter().any(|k| self.is_kind(k))
    }

    pub fn get_kind(&self) -> &Kind {
        &self.kind
    }
}
