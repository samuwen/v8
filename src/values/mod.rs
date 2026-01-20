mod abstracts;
mod number;
mod objects;
mod value;

use abstracts::*;
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
