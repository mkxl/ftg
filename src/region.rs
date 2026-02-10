use derive_more::Constructor;
use mkutils::Utils;
use nodit::{Interval, interval::InclusiveInterval};

#[derive(Clone, Constructor, Copy)]
pub struct Region {
    begin: usize,
    end: usize,
}

impl Region {
    pub const ZERO: Self = Self::new(0, 0);

    // pub fn is_reversed(&self) -> bool {
    //     self.end < self.begin
    // }
}

impl From<usize> for Region {
    fn from(offset: usize) -> Self {
        Self::new(offset, offset)
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
