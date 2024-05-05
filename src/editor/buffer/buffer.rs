use crate::{
    editor::buffer::search::SearchIter,
    utils::{any::Any, container::Identifiable, position::Position},
};
use derive_more::Constructor;
use ratatui::layout::Rect;
use ropey::Rope;
use std::{io::Error as IoError, path::Path};
use ulid::Ulid;

#[derive(Constructor)]
pub struct Buffer {
    id: Ulid,
    rope: Rope,
}

impl Buffer {
    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn from_filepath(filepath: &Path) -> Result<Self, IoError> {
        Self::new(filepath.inode_id()?, filepath.rope()?).ok()
    }

    pub fn search<'q, 'r>(&'r self, query: &'q str) -> SearchIter<'q, 'r> {
        SearchIter::new(&self.rope, query)
    }

    pub fn lines(&self, position: &Position, area: Rect) -> impl '_ + Iterator<Item = String> {
        // TODO: can i use this in the closure
        let begin = position.x;

        self.rope
            .get_lines_at(position.y)
            .into_iter()
            .flatten()
            .take(area.height as usize)
            .map(move |line| {
                let end = begin.saturating_add(area.width as usize).min(line.len_chars());
                let begin = begin.min(end);
                let line = line.slice(begin..end);

                line.to_string()
            })
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
        self.id()
    }
}
