use crate::utils::any::Any;
use nodit::{InclusiveInterval, Interval};

// NOTE: Copy impl needed for InclusiveInterval impl: [https://docs.rs/nodit/latest/nodit/interval/trait.InclusiveInterval.html]
#[derive(Clone, Copy)]
pub struct Region {
    begin: usize,
    last: usize,
    reversed: bool,
}

impl Region {
    fn new(begin: usize, last: usize, reversed: bool) -> Result<Self, Self> {
        if begin <= last {
            Self { begin, last, reversed }.ok()
        } else {
            Self {
                begin: last,
                last: begin,
                reversed,
            }
            .err()
        }
    }

    pub fn ii(begin: usize, last: usize) -> Self {
        Self::new(begin, last, false).into_inner()
    }

    pub fn try_ii(begin: usize, last: usize) -> Option<Self> {
        Self::new(begin, last, false).ok()
    }

    pub fn unit(begin: usize) -> Self {
        Self::ii(begin, begin)
    }

    pub fn try_ie(begin: usize, end: usize) -> Option<Self> {
        let last = end.saturating_sub(1);
        let result = Self::new(begin, last, false);

        result.ok()
    }

    pub fn begin(&self) -> usize {
        self.begin
    }

    pub fn last(&self) -> usize {
        self.last
    }

    pub fn reversed(&self) -> bool {
        self.reversed
    }

    pub fn end_exclusive(&self) -> usize {
        self.last().saturating_add(1)
    }

    pub fn len(&self) -> usize {
        self.end_exclusive().saturating_sub(self.begin())
    }

    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let begin = self.begin().max(other.begin());
        let last = self.last().min(other.last());

        Self::try_ii(begin, last)
    }

    pub fn try_with_begin(&self, begin: usize) -> Option<Self> {
        Self::new(begin, self.last(), self.reversed()).ok()
    }

    pub fn translate_by(&self, count: isize) -> Self {
        let begin = self.begin().saturating_add_signed(count);
        let last = self.last().saturating_add_signed(count);
        let result = Self::new(begin, last, self.reversed());

        result.into_inner()
    }
}

impl From<Interval<usize>> for Region {
    fn from(interval: Interval<usize>) -> Self {
        Self::ii(interval.start(), interval.end())
    }
}

impl InclusiveInterval<usize> for Region {
    fn start(&self) -> usize {
        self.begin()
    }

    fn end(&self) -> usize {
        self.last()
    }
}
