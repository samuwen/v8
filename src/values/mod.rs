mod number;
mod objects;
mod string;
mod value;

pub use number::*;
pub use objects::{JSObject, get_object_property, get_object_property_mut};
pub use value::*;

use crate::errors::JSError;

#[derive(Clone, Debug)]
pub enum PreferredType {
    String,
    Number,
}

pub type JSResult<T> = Result<T, JSError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectKind {
    Object,
    Function,
    Array,
}
