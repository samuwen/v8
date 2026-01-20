use std::collections::HashMap;

pub struct Heap<V: Clone> {
    map: HashMap<usize, V>,
    counter: usize,
}

impl<V: Clone> Heap<V> {
    pub fn new() -> Self {
        let map = HashMap::new();
        let counter = 0;
        Self { map, counter }
    }

    pub fn add_new_item(&mut self, item: V) -> usize {
        let object_id = self.counter;
        self.map.insert(object_id, item);
        self.counter += 1;
        object_id
    }

    pub fn get_item_from_id(&self, id: usize) -> V {
        // should be a safe unwrap - we only call this when we need to actually use a real item
        self.map.get(&id).unwrap().clone()
    }

    pub fn get_item_from_option(&self, opt: &Option<usize>) -> Option<V> {
        if let Some(proto_id) = opt {
            return Some(self.get_item_from_id(*proto_id));
        }
        None
    }
}
