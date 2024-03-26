use crate::{error::Error, utils::any::Any};
use derive_more::Constructor;
use itertools::Itertools;
use ropey::Rope;
use std::path::Path;

#[derive(Constructor, Default)]
pub struct Buffer {
    rope: Rope,
}

impl Buffer {
    pub fn from_filepath(filepath: &Path) -> Result<Self, Error> {
        let rope = filepath.rope()?;
        let buffer = Self::new(rope);

        buffer.ok()
    }

    pub fn lines(&self, begin: usize, count: usize) -> String {
        self.rope.get_lines_at(begin).into_iter().flatten().take(count).join("")
    }

    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }
}
