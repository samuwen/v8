use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use crate::span::Span;

#[derive(Debug, PartialEq, Clone)]
pub enum Kind {
    // Keywords
    Break,
    Case,
    Catch,
    Class,
    Const,
    Continue,
    Debugger,
    Default,
    Delete,
    Do,
    Else,
    Export,
    Extends,
    Finally,
    For,
    Function,
    If,
    Import,
    In,
    Instanceof,
    Let,
    New,
    Return,
    Super,
    Switch,
    This,
    Throw,
    Try,
    Typeof,
    Var,
    Void,
    While,
    With,
    Yield,

    // Future reserved words (strict mode)
    Await,
    Enum,
    Implements,
    Interface,
    Package,
    Private,
    Protected,
    Public,
    Static,

    // Literals
    True,
    False,
    Null,
    Undefined,
    Number,
    Identifier,
    String,

    // operators
    Plus,
    PlusPlus,
    PlusEquals,
    Minus,
    MinusMinus,
    Star,
    Slash,
    Equals,
    EqualEqual,
    EqualEqualEqual,
    Arrow,
    LeftParen,
    RightParen,
    LeftCurly,
    RightCurly,
    LeftSquare,
    RightSquare,
    Colon,
    Semicolon,
    Comma,
    NotEqual,
    Bang,
    LessThan,
    LessThanOrEquals,
    GreaterThan,
    GreaterThanOrEquals,
    Percent,
    Eof,
}

static KEYWORDS: OnceLock<Mutex<HashMap<&'static str, Kind>>> = OnceLock::new();

fn get_keywords() -> &'static Mutex<HashMap<&'static str, Kind>> {
    let mut m = HashMap::new();

    // Control flow
    m.insert("break", Kind::Break);
    m.insert("case", Kind::Case);
    m.insert("catch", Kind::Catch);
    m.insert("continue", Kind::Continue);
    m.insert("debugger", Kind::Debugger);
    m.insert("default", Kind::Default);
    m.insert("do", Kind::Do);
    m.insert("else", Kind::Else);
    m.insert("finally", Kind::Finally);
    m.insert("for", Kind::For);
    m.insert("if", Kind::If);
    m.insert("return", Kind::Return);
    m.insert("switch", Kind::Switch);
    m.insert("throw", Kind::Throw);
    m.insert("try", Kind::Try);
    m.insert("while", Kind::While);
    m.insert("with", Kind::With);

    // Declarations
    m.insert("class", Kind::Class);
    m.insert("const", Kind::Const);
    m.insert("function", Kind::Function);
    m.insert("let", Kind::Let);
    m.insert("var", Kind::Var);

    // Modules
    m.insert("export", Kind::Export);
    m.insert("import", Kind::Import);

    // Operators
    m.insert("delete", Kind::Delete);
    m.insert("in", Kind::In);
    m.insert("instanceof", Kind::Instanceof);
    m.insert("new", Kind::New);
    m.insert("typeof", Kind::Typeof);
    m.insert("void", Kind::Void);

    // Async/Generators
    m.insert("await", Kind::Await);
    m.insert("yield", Kind::Yield);

    // OOP
    m.insert("extends", Kind::Extends);
    m.insert("super", Kind::Super);
    m.insert("this", Kind::This);

    // Future reserved (strict mode)
    m.insert("enum", Kind::Enum);
    m.insert("implements", Kind::Implements);
    m.insert("interface", Kind::Interface);
    m.insert("package", Kind::Package);
    m.insert("private", Kind::Private);
    m.insert("protected", Kind::Protected);
    m.insert("public", Kind::Public);
    m.insert("static", Kind::Static);

    // Literals (technically not keywords but convenient to check)
    m.insert("true", Kind::True);
    m.insert("false", Kind::False);
    m.insert("null", Kind::Null);
    m.insert("undefined", Kind::Undefined);
    m.insert("infinity", Kind::Number);
    KEYWORDS.get_or_init(|| Mutex::new(m))
}

#[derive(Clone, Debug)]
pub struct Token {
    kind: Kind,
    span: Span,
}

impl Token {
    pub fn new(kind: Kind, line: usize, start: usize, end: usize) -> Self {
        Self {
            kind,
            span: Span::new(start, end, line),
        }
    }

    pub fn new_eof() -> Self {
        Self {
            kind: Kind::Eof,
            span: Span::new(0, 0, 0),
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

    pub fn get_span(&self) -> Span {
        self.span.clone()
    }

    pub fn is_binary_operator(&self) -> bool {
        match self.get_kind() {
            Kind::Plus
            | Kind::Minus
            | Kind::Slash
            | Kind::Star
            | Kind::LessThan
            | Kind::LessThanOrEquals
            | Kind::GreaterThan
            | Kind::GreaterThanOrEquals
            | Kind::Percent => true,
            _ => false,
        }
    }
}

pub fn get_keyword(word: &str) -> Option<Kind> {
    let map = get_keywords().lock().unwrap();
    map.get(word).map(|w| w.clone())
}
