use std::sync::OnceLock;

use log::trace;
use regex::Regex;
use string_interner::symbol::SymbolU32;

use crate::{Interpreter, errors::JSError, expr::Expr, values::JSResult};

static IDENTIFIER_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_ident_regex() -> &'static Regex {
    IDENTIFIER_REGEX.get_or_init(|| Regex::new("[a-zA-Z_$][a-zA-Z0-9_$]*").unwrap())
}

pub fn check_identifier(source: &str) -> JSResult<()> {
    trace!("checking identifier: {source}");
    let regex = get_ident_regex();
    if regex.is_match(source) {
        return Ok(());
    }
    Err(JSError::new("Identifier expected"))
}

pub fn get_function_params(
    args: &Vec<Expr>,
    interpreter: &mut Interpreter,
) -> JSResult<Vec<SymbolU32>> {
    let parameters = args
        .iter()
        .map(|arg| {
            let evaluated = arg.evaluate(interpreter)?;
            evaluated.to_string(interpreter)
        })
        .collect::<JSResult<Vec<SymbolU32>>>()?;
    Ok(parameters)
}

pub fn remove_quotes_from_string(string: &str) -> String {
    let single_quote = '\'';
    let double_quote = '"';
    string.chars().fold(String::new(), |mut acc, c| {
        if c != single_quote && c != double_quote {
            acc.push(c)
        }
        acc
    })
}
