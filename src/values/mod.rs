mod number;
mod objects;
mod value;

pub use number::*;
pub use objects::JSObject;
pub use value::*;

use crate::errors::JSError;

#[derive(Clone, Debug)]
pub enum PreferredType {
    String,
    Number,
}

pub type JSResult<T> = Result<T, JSError>;
