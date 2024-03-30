use crate::utils::{any::Any, container::Identifiable};
use derive_more::Constructor;
use ropey::{Rope, RopeSlice};
use std::{io::Error as IoError, path::Path};
use ulid::Ulid;

#[derive(Constructor)]
pub struct Buffer {
    id: Ulid,
    rope: Rope,
}

impl Buffer {
    pub fn from_filepath(filepath: &Path) -> Result<Self, IoError> {
        Self::new(filepath.inode_id()?, filepath.rope()?).ok()
    }

    pub fn lines(&self, begin: usize, count: usize) -> impl '_ + Iterator<Item = RopeSlice> {
        self.rope.get_lines_at(begin).into_iter().flatten().take(count)
    }

    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new(Ulid::new(), Rope::new())
    }
}

impl Identifiable for Buffer {
    fn id(&self) -> Ulid {
        self.id
    }
}
