use crate::rope::{
    chunk::{Chunk, ExtendedGraphemeIter},
    chunk_summary::{ExtendedGraphemeOffset, LineOffset},
};
use derive_more::Constructor;
use mkutils::Utils;
use std::{iter::Flatten, option::IntoIter as OptionIntoIter};
use zed_sum_tree::{Bias, Cursor, Dimensions, Item, SumTree, Summary};

type Dims = Dimensions<LineOffset, ExtendedGraphemeOffset>;

#[derive(Default)]
pub struct Rope {
    chunks: SumTree<Chunk>,
}

impl Rope {
    const CONTEXT: <<Chunk as Item>::Summary as Summary>::Context<'static> = ();

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_extended_graphemes(&mut self, string: &str) {
        let mut extended_graphemes = string.extended_graphemes().peekable();

        while extended_graphemes.peek().is_some() {
            // TODO: would ideally define [update_last] once, rather than each iteration,
            // but this introduces borrow issues
            let update_last = |last_chunk: &mut Chunk| {
                while let Some(extended_grapheme) = extended_graphemes.peek() {
                    if last_chunk.try_push_extended_grapheme(extended_grapheme).is_err() {
                        break;
                    }

                    extended_graphemes.next();
                }
            };

            self.chunks.update_last(update_last, Self::CONTEXT);
            self.chunks.push(Chunk::empty(), Self::CONTEXT);
        }
    }

    pub fn atom_iter(&self, line_offset: usize) -> AtomIter<'_> {
        AtomIter::new(self, line_offset)
    }
}

pub struct AtomIter<'r> {
    chunk_cursor: Cursor<'r, 'static, Chunk, Dims>,
    extended_grapheme_iter: Flatten<OptionIntoIter<ExtendedGraphemeIter<'r>>>,
    line_offset: usize,
    extended_grapheme_offset: usize,
}

impl<'r> AtomIter<'r> {
    const BIAS: Bias = Bias::Left;

    fn new(rope: &'r Rope, target_line_offset: usize) -> Self {
        let mut chunk_cursor = rope.chunks.cursor::<Dims>(Rope::CONTEXT);
        let target_line_offset_type = LineOffset::new(target_line_offset);

        chunk_cursor.seek(&target_line_offset_type, Self::BIAS);

        let (extended_grapheme_iter, line_offset, extended_grapheme_offset) =
            if let Some(chunk) = chunk_cursor.iter_next() {
                // NOTE: [line_offset] and [extended_grapheme_offset] here are the offsets for the start of the chunk
                let Dimensions(line_offset, extended_grapheme_offset, ()) = chunk_cursor.start();
                let mut line_offset = line_offset.inner();
                let mut extended_grapheme_offset = extended_grapheme_offset.inner();
                let mut extended_grapheme_iter = chunk.extended_grapheme_iter();

                while line_offset != target_line_offset {
                    extended_grapheme_offset.saturating_add_assign(&extended_grapheme_iter.advance_to_next_line());
                    line_offset.increment();
                }

                (extended_grapheme_iter.some(), line_offset, extended_grapheme_offset)
            } else {
                let rope_chunk_summary = rope.chunks.summary();

                (
                    None,
                    rope_chunk_summary.num_newlines(),
                    rope_chunk_summary.num_extended_graphemes(),
                )
            };

        let extended_grapheme_iter = extended_grapheme_iter.into_iter().flatten();

        Self {
            chunk_cursor,
            extended_grapheme_iter,
            line_offset,
            extended_grapheme_offset,
        }
    }
}

#[derive(Constructor)]
pub struct Atom<'r> {
    line_offset: usize,
    extended_grapheme_offset: usize,
    extended_grapheme: &'r str,
}

impl<'r> Iterator for AtomIter<'r> {
    type Item = Atom<'r>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(extended_grapheme) = self.extended_grapheme_iter.next() {
                self.extended_grapheme_offset.increment();

                return if extended_grapheme.is_newline() {
                    self.line_offset.increment().none()
                } else {
                    Atom::new(self.line_offset, self.extended_grapheme_offset, extended_grapheme).some()
                };
            }

            self.extended_grapheme_iter = self
                .chunk_cursor
                .iter_next()?
                .extended_grapheme_iter()
                .some()
                .into_iter()
                .flatten();
        }
    }
}
