use crate::region::Region;
use mkutils::{IntervalSet, Utils};

// NOTE: there is no direct [From<...>] implementation because we want to
// enforce that selections are non-empty
pub struct Selection {
    regions: IntervalSet<Region>,
}

impl Selection {
    fn new(region: Region) -> Self {
        let mut regions = IntervalSet::new();

        regions.insert(region);

        Self { regions }
    }

    pub fn from_regions(regions: impl IntoIterator<Item = Region>) -> Option<Self> {
        let mut regions = regions.into_iter();
        let mut selection = Self::new(regions.next()?);

        for region in regions {
            selection.insert(region);
        }

        selection.some()
    }

    pub fn contains_point(&self, point: usize) -> bool {
        self.regions.is_touching_or_contains(&point)
    }

    pub fn insert(&mut self, region: Region) -> &mut Self {
        self.regions.insert(region);

        self
    }

    pub fn translate(&mut self, count: isize) {
        for region in self.regions.mem_take() {
            let translated_region = region.translated(count);

            self.insert(translated_region);
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
