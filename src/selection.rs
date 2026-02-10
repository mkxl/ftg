use crate::region::Region;
use nodit::NoditSet;

// NOTE: there is no direct [From<...>] implementation because we want to
// enforce that selections are non-empty
pub struct Selection {
    regions: NoditSet<usize, Region>,
}

impl Selection {
    fn new(region: Region) -> Self {
        let mut regions = NoditSet::new();

        regions.insert_merge_touching_or_overlapping(region);

        Self { regions }
    }
}

impl From<Region> for Selection {
    fn from(region: Region) -> Self {
        Self::new(region)
    }
}
