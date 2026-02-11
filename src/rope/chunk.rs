use crate::rope::chunk_summary::ChunkSummary;
use arrayvec::{ArrayString, ArrayVec, CapacityError};
use mkutils::Utils;
use std::{ops::Range, slice::Iter};
use zed_sum_tree::{Item, Summary};

const CAPACITY: usize = 1024;

#[derive(Clone, Default)]
pub struct Chunk {
    string: ArrayString<CAPACITY>,
    extended_grapheme_offset_intervals: ArrayVec<Range<usize>, CAPACITY>,
    newline_offset_intervals: ArrayVec<Range<usize>, CAPACITY>,
}

impl Chunk {
    pub fn empty() -> Self {
        Self::default()
    }

    // TODO:
    // - it's potentially the case that the last codepoint in self and the argument
    //   extended_grapheme don't have a proper grapheme boundary; in that case, will
    //   need to repair the join boundary, without pushing to
    //   [self.extended_grapheme_offset_intervals]; see chatgpt
    //   - consider the strings "foo" (split into ["f", "o", "o"]) and "bar" (split
    //     into ["b", "a", "r"]) and assume the trailing "o" and the leading "b"
    //     don't have a proper boundary
    //   - then after pushing the trailing "o" and the leading "b" i should repair
    //     chunk so that the contained graphemes are ["f", "o", "ob", "a", "r"]
    //   - silly example, but could be the case if i'm reading strings piece by
    //     piece from a large file and the reader splits on code points?
    //   - in that case, should probably just have a note to make sure that
    //     [rope.push_str()] is only called with strings not split between extended
    //     graphemes
    //   - (***) could have my own rope builder that maintains a grapheme cursor(?)
    //     to ensure this doesn't happen
    pub fn try_push_extended_grapheme<'a>(&mut self, extended_grapheme: &'a str) -> Result<(), CapacityError<&'a str>> {
        let extended_grapheme_offset = self.string.len();
        let extended_grapheme_offset_interval = extended_grapheme_offset.range_from_len(extended_grapheme.len());

        // NOTE:
        // - [self.string] will always be weakly longer than
        //   [self.extended_grapheme_offset_intervals] (with equality only when each
        //   extended grapheme is a single byte) and because we only push to
        //   [self.extended_grapheme_offset_intervals] after successfully pushing to
        //   [self.string], we will never panic
        // - same holds for [self.newline_offset_intervals]
        self.string.try_push_str(extended_grapheme)?;
        self.extended_grapheme_offset_intervals
            .push(extended_grapheme_offset_interval.clone());

        if extended_grapheme.is_newline() {
            self.newline_offset_intervals.push(extended_grapheme_offset_interval);
        }

        ().ok()
    }

    pub fn extended_grapheme_iter(&self) -> ExtendedGraphemeIter<'_> {
        ExtendedGraphemeIter::new(self)
    }
}

impl Item for Chunk {
    type Summary = ChunkSummary;

    fn summary(&self, _context: <Self::Summary as Summary>::Context<'_>) -> Self::Summary {
        ChunkSummary::new(
            self.extended_grapheme_offset_intervals.len(),
            self.newline_offset_intervals.len(),
        )
    }
}

pub struct ExtendedGraphemeIter<'c> {
    string: &'c ArrayString<CAPACITY>,
    extended_grapheme_offset_interval_iter: Iter<'c, Range<usize>>,
}

impl<'c> ExtendedGraphemeIter<'c> {
    fn new(chunk: &'c Chunk) -> Self {
        let string = &chunk.string;
        let extended_grapheme_offset_interval_iter = chunk.extended_grapheme_offset_intervals.iter();

        Self {
            string,
            extended_grapheme_offset_interval_iter,
        }
    }
}

impl<'c> Iterator for ExtendedGraphemeIter<'c> {
    type Item = &'c str;

    fn next(&mut self) -> Option<Self::Item> {
        self.string[self.extended_grapheme_offset_interval_iter.next()?.clone()]
            .ref_immut()
            .some()
    }
}
