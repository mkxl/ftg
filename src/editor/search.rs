use crate::{editor::selection::region::Region, utils::any::Any};
use ropey::{iter::Chars as RopeChars, Rope};
use std::{
    iter::{Enumerate, Peekable},
    str::Chars as StrChars,
};

struct Query<'a> {
    begin: usize,
    chars: Peekable<StrChars<'a>>,
}

impl<'a> Query<'a> {
    fn new(begin: usize, text: &'a str) -> Self {
        Self {
            begin,
            chars: text.chars().peekable(),
        }
    }
}

pub struct SearchIter<'a> {
    queries: Vec<Query<'a>>,
    query: &'a str,
    rope_iter: Enumerate<RopeChars<'a>>,
}

impl<'a> SearchIter<'a> {
    fn new(rope: &'a Rope, query: &'a str) -> Self {
        Self {
            queries: std::vec![],
            query,
            rope_iter: rope.chars().enumerate(),
        }
    }
}

impl<'a> Iterator for SearchIter<'a> {
    type Item = Region;

    fn next(&mut self) -> Option<Self::Item> {
        for (idx, rope_char) in &mut self.rope_iter {
            // NOTE:
            // - need to remove items from self.queries as we iterate over it
            // - easiest way to do that is to keep a pointer to current position, and call .swap_remove() whenever
            //   an element needs to be removed or increment the pointer otherwise
            let query = Query::new(idx, self.query);
            let mut i = 0;

            // NOTE: possible matches starting at `idx`
            self.queries.push(query);

            while i < self.queries.len() {
                let query = &mut self.queries[i];
                let query_char = query.chars.next();
                let next_query_char = query.chars.peek();
                let begin = query.begin;

                match (query_char == Some(rope_char), next_query_char) {
                    // NOTE: possible is confimed as a match
                    (true, None) => {
                        self.queries.swap_remove(i);

                        return (begin, idx).convert::<Region>().some();
                    }

                    // NOTE: query_chars is still valid, but has not been confirmed as a match
                    (true, _) => {
                        i += 1;

                        continue;
                    }

                    // NOTE: query_chars is no longer valid and needs to be removed
                    (false, _) => self.queries.swap_remove(i).unit(),
                }
            }
        }

        None
    }
}
