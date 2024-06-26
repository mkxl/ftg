use crate::error::Error;
use std::collections::{hash_map::Entry, HashMap};
use ulid::Ulid;

macro_rules! get_impl {
    ($self:ident, $method:ident, $id:ident) => {{
        $self
            .values
            .$method($id)
            .ok_or_else(|| Error::UnknownItem($self.name.clone(), *$id))
    }};
}

pub trait Identifiable {
    fn id(&self) -> Ulid;
}

pub struct Container<T> {
    name: String,
    values: HashMap<Ulid, T>,
}

impl<T> Container<T> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            values: HashMap::new(),
        }
    }
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

    pub fn get(&self, id: &Ulid) -> Result<&T, Error> {
        get_impl!(self, get, id)
    }

    pub fn get_mut(&mut self, id: &Ulid) -> Result<&mut T, Error> {
        get_impl!(self, get_mut, id)
    }
}
