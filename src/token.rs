use string_interner::symbol::SymbolU32;

#[derive(Debug, PartialEq, Clone)]
pub enum Kind {
    // Literals
    Number(f64),
    String(SymbolU32),     // index to string map
    Identifier(SymbolU32), // index to string map
    True,
    False,
    Null,
    Undefined,

    // Keywords
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
    For,

    // Operators
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
    LeftCurly,
    RightCurly,
    LeftSquare,
    RightSquare,
    Arrow,
    Colon,
    Semicolon,
    Comma,

    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    kind: Kind,
    _line: usize,
    _start: usize,
    _end: usize,
}

impl Token {
    pub fn new(kind: Kind, line: usize, start: usize, end: usize) -> Self {
        Self {
            kind,
            _line: line,
            _start: start,
            _end: end,
        }
    }

    pub fn new_eof() -> Self {
        Self {
            kind: Kind::Eof,
            _line: 0,
            _start: 0,
            _end: 0,
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
