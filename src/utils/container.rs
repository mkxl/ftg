use std::collections::HashMap;
use ulid::Ulid;

pub trait Identifiable {
    fn id(&self) -> Ulid;
}

pub struct Container<T> {
    values: HashMap<Ulid, T>,
}

impl<T: Identifiable> Container<T> {
    pub fn insert(&mut self, value: T) {
        self.values.insert(value.id(), value);
    }

    pub fn get_mut(&mut self, id: &Ulid) -> Option<&mut T> {
        self.values.get_mut(id)
    }
}

impl<T> Default for Container<T> {
    fn default() -> Self {
        Self {
            values: HashMap::default(),
        }
    }
}
