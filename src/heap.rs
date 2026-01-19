use std::collections::HashMap;

use crate::values::JSObject;

pub struct Heap {
    map: HashMap<usize, JSObject>,
    counter: usize,
}

impl Heap {
    pub fn new() -> Self {
        let map = HashMap::new();
        let counter = 0;
        Self { map, counter }
    }

    pub fn add_new_object(&mut self, object: JSObject) -> usize {
        let object_id = self.counter;
        self.map.insert(object_id, object);
        self.counter += 1;
        object_id
    }

    pub fn get_object_from_id(&self, id: usize) -> JSObject {
        // should be a safe unwrap - we only call this when we need to actually use a real object
        self.map.get(&id).unwrap().clone()
    }

    pub fn get_object_from_option(&self, opt: &Option<usize>) -> Option<JSObject> {
        if let Some(proto_id) = opt {
            return Some(self.get_object_from_id(*proto_id));
        }
        None
    }
}
