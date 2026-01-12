use crate::string_pool::StringPool;

#[derive(Debug, PartialEq, Clone)]
pub enum Kind {
    // Literals
    Number(f64),
    String(usize),     // index to string map
    Identifier(usize), // index to string map

    // Keywords (start with just these)
    Var,
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
    EqualEqualEqual,

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
}
