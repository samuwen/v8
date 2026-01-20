use crate::counter::Counter;

#[derive(Debug, Clone)]
pub struct Environment {
    id: u32,
    parent_id: Option<u32>,
    handles: Vec<u32>,
}

impl Environment {
    pub fn new_root() -> Self {
        Self {
            id: 0,
            parent_id: None,
            handles: vec![],
        }
    }

    pub fn new(parent_id: u32, counter: &mut Counter) -> Self {
        let id = counter.get();
        Self {
            id,
            parent_id: Some(parent_id),
            handles: vec![],
        }
    }

    pub fn add_handle(&mut self, handle: u32) {
        self.handles.push(handle);
    }

    pub fn get_handles(&self) -> &Vec<u32> {
        &self.handles
    }
}
