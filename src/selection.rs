use crate::region::Region;
use mkutils::Utils;
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

    pub fn contains_point(&self, point: usize) -> bool {
        self.regions.contains_point(&point)
    }

    pub fn translate(&mut self, count: isize) {
        let original_regions = self.regions.mem_take();

        for region in original_regions {
            let translated_region = region.translated(count);

            self.regions.insert_merge_touching_or_overlapping(translated_region);
        }
    }

    pub fn regions(&self) -> impl DoubleEndedIterator<Item = &Region> {
        self.regions.iter()
    }
}

impl From<Region> for Selection {
    fn from(region: Region) -> Self {
        Self::new(region)
    }
}
