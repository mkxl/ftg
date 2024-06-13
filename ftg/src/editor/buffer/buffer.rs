use crate::{
    editor::{buffer::search::SearchIter, selection::region::Region},
    utils::{any::Any, container::Identifiable, position::Position},
};
use derive_more::Constructor;
use ratatui::layout::Rect;
use ropey::{iter::Chunks, Error as RopeyError, Rope, RopeSlice};
use std::{io::Error as IoError, path::Path};
use ulid::Ulid;
use unicode_segmentation::UnicodeSegmentation;

pub struct LineCharIndices {
    pub begin: usize,
    pub last: usize,
    pub query: usize,
}

#[derive(Constructor)]
pub struct SubLine<'a> {
    slice: RopeSlice<'a>,
    region: Option<Region>,
}

impl<'a> SubLine<'a> {
    pub fn region(&self) -> Option<Region> {
        self.region
    }

    pub fn graphemes(&self) -> impl Iterator<Item = &str> {
        self.slice.chunks().flat_map(|chunk_str| chunk_str.graphemes(true))
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
        // TODO-c8394f:
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
                let begin = line_char_idx.saturating_add(begin);
                let end = line_char_idx.saturating_add(end);
                let region = Region::try_ie(begin, end);
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

    pub fn row_col(&self, char_idx: usize) -> (usize, usize) {
        // TODO-9ec981:
        // - figure out if this is the most efficient way to do this (calling multiple different rope methods)
        // - TODO-c8394f: feels like this would be obviated if we could get the real index associated w a sub-rope-slice
        let row = self.rope.char_to_line(char_idx);
        let char_idx_of_line = self.rope.line_to_char(row);
        let col = char_idx.saturating_sub(char_idx_of_line);

        (row, col)
    }

    // NOTE:
    // - row will saturate at the max possible row
    // - col will saturate at the max possible col for the given row
    pub fn char_idx(&self, row: usize, col: usize) -> LineCharIndices {
        // TODO-9ec981
        let max_row = self.rope.len_lines().saturating_sub(1);
        let row = row.clamp(0, max_row);
        let char_idx_of_line_begin = self.rope.line_to_char(row);
        let line_rope_slice = self.rope.line(row);
        let max_col = line_rope_slice.len_chars().saturating_sub(1);
        let col = col.clamp(0, max_col);
        let char_idx = char_idx_of_line_begin.saturating_add(col);
        let char_idx_of_line_last = char_idx.saturating_add(col);

        LineCharIndices {
            begin: char_idx_of_line_begin,
            last: char_idx_of_line_last,
            query: char_idx,
        }
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
