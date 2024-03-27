use crate::{error::Error, utils::any::Any};
use derive_more::Constructor;
use itertools::Itertools;
use ropey::Rope;
use std::{os::unix::fs::MetadataExt, path::Path};
use ulid::Ulid;

#[derive(Constructor)]
pub struct Buffer {
    id: Ulid,
    rope: Rope,
}

impl Buffer {
    pub fn from_filepath(filepath: &Path) -> Result<Self, Error> {
        let id = filepath.metadata()?.ino().convert::<u128>().into();
        let rope = filepath.rope()?;
        let buffer = Self::new(id, rope);

        buffer.ok()
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn lines(&self, begin: usize, count: usize) -> String {
        self.rope.get_lines_at(begin).into_iter().flatten().take(count).join("")
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
