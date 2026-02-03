#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashMap;

pub struct Heap<V> {
    map: HashMap<usize, V>,
    counter: usize,
}

impl<V> Heap<V> {
    pub fn new() -> Self {
        let map = HashMap::new();
        let counter = 0;
        Self { map, counter }
    }

    pub fn add_new_item(&mut self, item: V) -> usize {
        let item_id = self.increment();
        self.map.insert(item_id, item);
        item_id
    }

    pub fn add_new_item_with_id(&mut self, item: V, id: usize) -> usize {
        self.map.insert(id, item);
        id
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut V> {
        self.map.get_mut(&id)
    }

    pub fn get_item_from_id(&self, id: usize) -> Option<&V> {
        self.map.get(&id)
    }

    pub fn get_item_from_option(&mut self, opt: &Option<usize>) -> Option<&mut V> {
        if let Some(proto_id) = opt {
            return self.get_mut(*proto_id);
        }
        None
    }

    pub fn get_next_id(&mut self) -> usize {
        self.increment()
    }

    pub fn has_item(&self, id: usize) -> bool {
        self.map.contains_key(&id)
    }

    fn increment(&mut self) -> usize {
        let id = self.counter;
        self.counter += 1;
        id
    }
}
