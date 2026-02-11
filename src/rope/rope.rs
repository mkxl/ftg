use crate::rope::{
    chunk::{Chunk, ExtendedGraphemeIter},
    chunk_summary::{ChunkSummary, LineOffset},
};
use mkutils::Utils;
use std::{iter::Flatten, option::IntoIter as OptionIntoIter};
use zed_sum_tree::{Bias, Cursor, Dimensions, Item, SumTree, Summary};

type Dims = Dimensions<LineOffset, ChunkSummary>;

// fn foo<'a>(_x: impl zed_sum_tree::SeekTarget<'a, ChunkSummary, Dims>) {}

// fn bar() {
//     foo(LineOffset::new(0))
// }

#[derive(Default)]
pub struct Rope {
    chunks: SumTree<Chunk>,
}

impl Rope {
    const BIAS: Bias = Bias::Left;
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

    pub fn atoms_at_line(&self, line_offset: usize) -> Iter<'_> {
        std::todo!()

        // let chunk_cursor = self.chunks.cursor(Self::CONTEXT);
        // let line_offset = LineOffset::new(line_offset);

        // chunk_cursor.seek(&line_offset, Self::BIAS);

        // Iter::new(chunk_cursor)
    }
}

pub struct Iter<'r> {
    chunk_cursor: Cursor<'r, 'static, Chunk, Dims>,
    extended_grapheme_iter: Flatten<OptionIntoIter<ExtendedGraphemeIter<'r>>>,
}

impl<'r> Iter<'r> {
    fn new(mut chunk_cursor: Cursor<'r, 'static, Chunk, Dims>) -> Self {
        let extended_grapheme_iter = chunk_cursor
            .iter_next()
            .map(Chunk::extended_grapheme_iter)
            .into_iter()
            .flatten();

        Self {
            chunk_cursor,
            extended_grapheme_iter,
        }
    }
}

pub struct Atom<'r> {
    line_offset: usize,
    extended_grapheme_offset: usize,
    extended_grapheme: &'r str,
}

impl<'r> Iterator for Iter<'r> {
    type Item = Atom<'r>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(extended_grapheme) = self.extended_grapheme_iter.next() {
                return if extended_grapheme.is_newline() {
                    None
                } else {
                    std::todo!("convert extended_grapheme to an atom")
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
