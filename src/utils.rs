use std::sync::OnceLock;

use log::debug;
use regex::Regex;

use crate::{errors::JSError, values::JSResult};

static IDENTIFIER_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_ident_regex() -> &'static Regex {
    IDENTIFIER_REGEX.get_or_init(|| Regex::new("[a-zA-Z_$][a-zA-Z0-9_$]*").unwrap())
}

pub fn check_identifier(source: &str) -> JSResult<()> {
    debug!("checking identifier: {source}");
    let regex = get_ident_regex();
    if regex.is_match(source) {
        return Ok(());
    }
    Err(JSError::new("Identifier expected"))
}
