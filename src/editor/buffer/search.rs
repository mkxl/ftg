use crate::{editor::selection::selection::Region, utils::any::Any};
use ropey::{iter::Chars as RopeChars, Rope};
use std::{
    iter::{Enumerate, Peekable},
    str::Chars as StrChars,
};

struct PossibleMatch<'q> {
    begin: usize,
    chars: Peekable<StrChars<'q>>,
}

impl<'q> PossibleMatch<'q> {
    fn new(begin: usize, text: &'q str) -> Self {
        Self {
            begin,
            chars: text.chars().peekable(),
        }
    }
}

pub struct SearchIter<'q, 'r> {
    possible_matches: Vec<PossibleMatch<'q>>,
    query: &'q str,
    rope_iter: Enumerate<RopeChars<'r>>,
}

impl<'q, 'r> SearchIter<'q, 'r> {
    pub fn new(rope: &'r Rope, query: &'q str) -> Self {
        Self {
            possible_matches: std::vec![],
            query,
            rope_iter: rope.chars().enumerate(),
        }
    }
}

impl<'q, 'r> Iterator for SearchIter<'q, 'r> {
    type Item = Region;

    fn next(&mut self) -> Option<Self::Item> {
        for (idx, rope_char) in &mut self.rope_iter {
            // NOTE:
            // - need to remove items from self.possible_matches as we iterate over it
            // - easiest way to do that is to keep a pointer to current position, and call .swap_remove() whenever
            //   an element needs to be removed or increment the pointer otherwise
            let possible_match = PossibleMatch::new(idx, self.query);
            let mut i = 0;

            // NOTE: possible matches starting at `idx`
            self.possible_matches.push(possible_match);

            while i < self.possible_matches.len() {
                let possible_match = &mut self.possible_matches[i];
                let possible_match_char = possible_match.chars.next();
                let next_possible_match_char = possible_match.chars.peek();
                let begin = possible_match.begin;

                match (possible_match_char == Some(rope_char), next_possible_match_char) {
                    // NOTE: possible is confimed as a match
                    (true, None) => {
                        self.possible_matches.swap_remove(i);

                        #[allow(clippy::range_plus_one)]
                        return (begin..(idx + 1)).some();
                    }

                    // NOTE: query_chars is still valid, but has not been confirmed as a match
                    (true, _) => {
                        i += 1;

                        continue;
                    }

                    // NOTE: query_chars is no longer valid and needs to be removed
                    (false, _) => self.possible_matches.swap_remove(i).unit(),
                }
            }
        }

        None
    }
}
