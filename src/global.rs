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
