use derive_more::Constructor;
use mkutils::Utils;
use nodit::{Interval, interval::InclusiveInterval};
use std::ops::Range;

#[derive(Clone, Constructor, Copy)]
pub struct Region {
    begin: usize,
    end: usize,
}

impl Region {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn from_offset(offset: usize) -> Self {
        Self::new(offset, offset)
    }

    pub fn min(&self) -> usize {
        self.start().copied()
    }

    pub fn max(&self) -> usize {
        self.end().copied()
    }

    pub const fn translated(&self, count: isize) -> Self {
        let begin = self.begin.saturating_add_signed(count);
        let end = self.end.saturating_add_signed(count);

        Self::new(begin, end)
    }

    pub fn translate(&mut self, count: isize) {
        self.translated(count).assign_to(self);
    }

    pub fn range(&self) -> Range<usize> {
        self.start().copied()..self.end().incremented()
    }
}

impl From<usize> for Region {
    fn from(offset: usize) -> Self {
        Self::from_offset(offset)
    }
}

impl From<Interval<usize>> for Region {
    fn from(interval: Interval<usize>) -> Self {
        Self::new(*interval.start(), *interval.end())
    }
}

impl InclusiveInterval<usize> for Region {
    fn start(&self) -> &usize {
        self.begin.ref_immut().min(&self.end)
    }

    fn end(&self) -> &usize {
        self.begin.ref_immut().max(&self.end)
    }
}
