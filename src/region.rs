use derive_more::Constructor;
use mkutils::{Interval, Utils};
use num::traits::ConstZero;
use std::ops::Range;

#[derive(Constructor)]
pub struct Region {
    begin: usize,
    len: usize,
    is_forward: bool,
}

impl Region {
    pub const ZERO: Self = Self::from_offset(usize::ZERO);

    const INITIAL_LEN: usize = usize::ZERO;
    const INITIAL_IS_FORWARD: bool = true;

    pub const fn from_offset(begin: usize) -> Self {
        Self::new(begin, Self::INITIAL_LEN, Self::INITIAL_IS_FORWARD)
    }

    pub const fn translated(&self, count: isize) -> Self {
        let begin = self.begin.saturating_add_signed(count);

        Self::new(begin, self.len, self.is_forward)
    }

    #[expect(dead_code)]
    pub fn translate(&mut self, count: isize) {
        self.translated(count).assign_to(self);
    }

    pub const fn expanded(&self, count: isize) -> Self {
        let len = self.len.saturating_add_signed(count);

        Self::new(self.begin, len, self.is_forward)
    }

    #[expect(dead_code)]
    pub fn expand(&mut self, count: isize) {
        self.expanded(count).assign_to(self);
    }

    pub fn range(&self) -> Range<usize> {
        self.begin.range_from_len(self.len)
    }
}

impl From<usize> for Region {
    fn from(offset: usize) -> Self {
        Self::from_offset(offset)
    }
}

impl Interval for Region {
    type Point = usize;

    fn begin(&self) -> Self::Point {
        self.begin
    }

    fn len(&self) -> Self::Point {
        self.len
    }

    fn expand_to_cover(&mut self, other: &Self) {
        let new_begin = self.begin().min(other.begin());
        let new_end = self.end().max(other.end());
        let new_len = new_end.saturating_sub(new_begin);

        self.begin = new_begin;
        self.len = new_len;
    }
}
