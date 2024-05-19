use crate::{
    editor::{buffer::search::SearchIter, selection::region::Region},
    utils::{any::Any, container::Identifiable, position::Position},
};
use derive_more::Constructor;
use ratatui::layout::Rect;
use ropey::{
    iter::{Chars, Chunks},
    Error as RopeyError, Rope, RopeSlice,
};
use std::{io::Error as IoError, path::Path};
use ulid::Ulid;

#[derive(Constructor)]
pub struct SubLine<'a> {
    slice: RopeSlice<'a>,
    region: Option<Region>,
}

impl<'a> SubLine<'a> {
    pub fn chars(&self) -> Chars {
        self.slice.chars()
    }

    pub fn region(&self) -> Option<Region> {
        self.region
    }
}

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

    pub fn sub_lines<'a>(&'a self, position: &'a Position, area: Rect) -> impl 'a + Iterator<Item = SubLine<'a>> {
        // TODO:
        // - is there a more efficient way of getting the char_idx of the position.y-th line?
        // - i don't like that i need to call .line_to_char() bc .get_lines_at() doesn't contain that information bc
        //   rope slices themselves don't contain that information (publicly afaict)
        let mut line_char_idx = self.rope.line_to_char(position.y);
        let begin = position.x;

        // NOTE:
        // - .into_iter().flatten() to flatten the Option return by .get_lines_at()
        // - let begin = begin.min(end) in case end has been reduced to 0 because len_chars is 0
        self.rope
            .get_lines_at(position.y)
            .into_iter()
            .flatten()
            .take(area.height as usize)
            .map(move |line_slice| {
                let len_chars = line_slice.len_chars();
                let end = begin.saturating_add(area.width as usize).min(len_chars);
                let begin = begin.min(end);
                let slice = line_slice.slice(begin..end);
                let region = Region::ie(line_char_idx + begin, line_char_idx + end);

                let sub_line = SubLine::new(slice, region);

                line_char_idx += len_chars;

                sub_line
            })
    }

    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn insert_char(&mut self, char_idx: usize, chr: char) -> Result<(), RopeyError> {
        self.rope.try_insert_char(char_idx, chr)
    }

    pub fn chunks(&self) -> Chunks {
        self.rope.chunks()
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