use crate::utils::any::Any;
use derive_more::Constructor;
use nodit::{InclusiveInterval, Interval};

#[derive(Clone, Constructor, Copy)]
pub struct Region {
    interval: Interval<usize>,
    reversed: bool,
}

impl Region {
    fn from_values(start: usize, end: usize, reversed: bool) -> Option<Self> {
        if start <= end {
            Self::new(nodit::interval::ii(start, end), reversed).some()
        } else {
            None
        }
    }

    pub fn unit(start: usize) -> Self {
        Self::ii(start, start).unwrap()
    }

    pub fn ii(start: usize, end: usize) -> Option<Self> {
        Self::from_values(start, end, false)
    }

    pub fn ie(start: usize, end_exclusive: usize) -> Option<Self> {
        Self::ii(start, end_exclusive.saturating_sub(1))
    }

    pub fn start(&self) -> usize {
        self.interval.start()
    }

    pub fn end(&self) -> usize {
        self.interval.end()
    }

    pub fn end_exclusive(&self) -> usize {
        self.end().saturating_add(1)
    }

    pub fn reversed(&self) -> bool {
        self.reversed
    }

    pub fn len(&self) -> usize {
        self.end_exclusive().saturating_sub(self.start())
    }

    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let start = self.start().max(other.start());
        let end = self.end().min(other.end());

        Self::ii(start, end)
    }

    pub fn with_start(&self, start: usize) -> Option<Self> {
        Self::from_values(start, self.end(), self.reversed())
    }

    pub fn translate_by(&self, count: isize) -> Self {
        // TODO: see if i can use InclusiveInterval::translate; not using immediately bc of usize/isize business w
        // std::ops::Add
        let interval = nodit::interval::ii(
            self.start().saturating_add_signed(count),
            self.end().saturating_add_signed(count),
        );

        Self {
            interval,
            reversed: self.reversed(),
        }
    }
}

impl From<Interval<usize>> for Region {
    fn from(interval: Interval<usize>) -> Self {
        Self::new(interval, false)
    }
}

impl InclusiveInterval<usize> for Region {
    fn start(&self) -> usize {
        self.start()
    }

    fn end(&self) -> usize {
        self.end()
    }
}
