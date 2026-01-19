use std::collections::HashMap;

use string_interner::symbol::SymbolU32;

use crate::values::object::JSObject;

pub struct OrdinaryObject<'o> {
    prototype: Option<&'o JSObject>,
    property_map: HashMap<SymbolU32, JSObject>,
}
