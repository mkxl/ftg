use crate::rope::chunk_summary::ChunkSummary;
use arrayvec::{ArrayString, ArrayVec, CapacityError};
use mkutils::Utils;
use std::ops::Range;
use zed_sum_tree::{Item, Summary};

const CAPACITY: usize = 1024;

#[derive(Clone, Default)]
pub struct Chunk {
    string: ArrayString<CAPACITY>,
    extended_grapheme_intervals: ArrayVec<Range<usize>, CAPACITY>,
    extended_grapheme_interval_index_from_newline_index: ArrayVec<usize, CAPACITY>,
}

impl Chunk {
    pub fn empty() -> Self {
        Self::default()
    }

    // TODO:
    // - it's potentially the case that the last codepoint in self and the argument extended_grapheme don't have a
    //   proper grapheme boundary; in that case, will need to repair the join boundary, without pushing to
    //   [self.extended_grapheme_intervals]; see chatgpt
    //   - consider the strings "foo" (split into ["f", "o", "o"]) and "bar" (split into ["b", "a", "r"]) and assume the
    //     trailing "o" and the leading "b" don't have a proper boundary
    //   - then after pushing the trailing "o" and the leading "b" i should repair chunk so that the contained graphemes
    //     are ["f", "o", "ob", "a", "r"]
    //   - silly example, but could be the case if i'm reading strings piece by piece from a large file and the reader
    //     splits on code points?
    //   - in that case, should probably just have a note to make sure that [rope.push_str()] is only called with
    //     strings not split between extended graphemes
    //   - (***) could have my own rope builder that maintains a grapheme cursor(?) to ensure this doesn't happen
    pub fn try_push_extended_grapheme<'a>(&mut self, extended_grapheme: &'a str) -> Result<(), CapacityError<&'a str>> {
        let extended_grapheme_interval_begin = self.string.len();
        let extended_grapheme_interval_index = self.extended_grapheme_intervals.len();

        // NOTE:
        // - [self.string] will always be weakly longer than [self.extended_grapheme_intervals] (with equality only when
        //   each extended grapheme is a single byte) and because we only push to [self.extended_grapheme_intervals]
        //   after successfully pushing to [self.string], we will never panic
        // - same holds for [self.extended_grapheme_interval_index_from_newline_index]
        self.string.try_push_str(extended_grapheme)?;
        extended_grapheme_interval_begin
            .range_from_len(extended_grapheme.len())
            .push_to(&mut self.extended_grapheme_intervals);

        if extended_grapheme.is_newline() {
            self.extended_grapheme_interval_index_from_newline_index
                .push(extended_grapheme_interval_index);
        }

        ().ok()
    }

    fn num_extended_graphemes(&self) -> usize {
        self.extended_grapheme_intervals.len()
    }

    fn num_newlines(&self) -> usize {
        self.extended_grapheme_interval_index_from_newline_index.len()
    }

    pub fn extended_grapheme_iter(&self) -> ExtendedGraphemeIter<'_> {
        ExtendedGraphemeIter::new(self)
    }
}

impl Item for Chunk {
    type Summary = ChunkSummary;

    fn summary(&self, _context: <Self::Summary as Summary>::Context<'_>) -> Self::Summary {
        ChunkSummary::new(self.num_extended_graphemes(), self.num_newlines())
    }
}

pub struct ExtendedGraphemeIter<'c> {
    chunk: &'c Chunk,
    extended_grapheme_interval_index: usize,
}

impl<'c> ExtendedGraphemeIter<'c> {
    const INITIAL_EXTENDED_GRAPHEME_INTERVAL_INDEX: usize = 0;

    fn new(chunk: &'c Chunk) -> Self {
        let extended_grapheme_interval_index = Self::INITIAL_EXTENDED_GRAPHEME_INTERVAL_INDEX;

        Self {
            chunk,
            extended_grapheme_interval_index,
        }
    }

    pub fn advance_to_next_line(&mut self) -> usize {
        let extended_grapheme_interval_indices_of_newlines = self
            .chunk
            .extended_grapheme_interval_index_from_newline_index
            .iter()
            .copied();

        for extended_grapheme_interval_index_of_newline in extended_grapheme_interval_indices_of_newlines {
            if self.extended_grapheme_interval_index < extended_grapheme_interval_index_of_newline {
                let new_extended_grapheme_interval_index =
                    extended_grapheme_interval_index_of_newline.saturating_add(1);

                return self
                    .extended_grapheme_interval_index
                    .translate_to(new_extended_grapheme_interval_index);
            }
        }

        // NOTE: if there is no next line, set [self.extended_grapheme_interval_index] to point to the end of
        // [self.chunk.extended_grapheme_intervals]
        self.extended_grapheme_interval_index
            .translate_to(self.chunk.extended_grapheme_intervals.len())
    }
}

impl<'c> Iterator for ExtendedGraphemeIter<'c> {
    type Item = &'c str;

    fn next(&mut self) -> Option<Self::Item> {
        let extended_grapheme = self
            .chunk
            .extended_grapheme_intervals
            .get(self.extended_grapheme_interval_index)?
            .clone()
            .index_into(self.chunk.string.as_str());

        self.extended_grapheme_interval_index.increment();

        extended_grapheme.some()
    }
}
