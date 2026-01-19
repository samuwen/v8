use crate::span::Span;
use crate::values::JSObject;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;
use string_interner::{StringInterner, backend::StringBackend, symbol::SymbolU32};

static STRING_INTERNER: OnceLock<Mutex<StringInterner<StringBackend>>> = OnceLock::new();

fn get_interner() -> &'static Mutex<StringInterner<StringBackend>> {
    STRING_INTERNER.get_or_init(|| Mutex::new(StringInterner::new()))
}

pub fn get_or_intern_string(s: &str) -> SymbolU32 {
    get_interner().lock().unwrap().get_or_intern(s)
}

pub fn get_string_from_pool(sym: &SymbolU32) -> Option<String> {
    let interner = get_interner().lock().unwrap();
    interner.resolve(*sym).map(|s| s.to_owned())
}

static SOURCE_STRING: OnceLock<Mutex<String>> = OnceLock::new();

fn get_source_string() -> &'static Mutex<String> {
    SOURCE_STRING.get_or_init(|| Mutex::new(String::new()))
}

pub fn set_source_string(s: &str) {
    let mut source = get_source_string().lock().unwrap();
    source.clear();
    source.push_str(s);
}

pub fn _get_full_source() -> String {
    let source = get_source_string().lock().unwrap();
    source.to_owned()
}

pub fn get_source_at_span(span: &Span) -> String {
    let source = get_source_string().lock().unwrap();
    let result = &source[span.get_as_range()];
    result.to_string()
}
