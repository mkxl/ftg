use std::collections::{hash_map::Entry, HashMap};
use ulid::Ulid;

pub trait Identifiable {
    fn id(&self) -> Ulid;
}

pub struct Container<T> {
    values: HashMap<Ulid, T>,
}

impl<T: Identifiable> Container<T> {
    pub fn insert(&mut self, value: T) -> &mut T {
        match self.values.entry(value.id()) {
            Entry::Occupied(mut occupied_entry) => {
                occupied_entry.insert(value);

                occupied_entry.into_mut()
            }
            Entry::Vacant(vacant_entry) => vacant_entry.insert(value),
        }
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
