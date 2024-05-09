use crate::utils::any::Any;
use derive_more::From;
use rangemap::{set::Iter as RangeSetIter, RangeSet};
use std::ops::Range;

pub type Region = Range<usize>;
pub type RegionSet = RangeSet<usize>;
pub type RegionSetIter<'a> = RangeSetIter<'a, usize>;

#[derive(From)]
pub struct Selection {
    regions: RegionSet,
}

impl Selection {
    pub fn iter(&self) -> RegionSetIter {
        return self.regions.iter();
    }
}

impl From<Region> for Selection {
    fn from(region: Region) -> Self {
        region.once().collect()
    }
}

impl FromIterator<Region> for Selection {
    fn from_iter<T: IntoIterator<Item = Region>>(iter: T) -> Self {
        iter.into_iter().collect::<RegionSet>().into()
    }
}
