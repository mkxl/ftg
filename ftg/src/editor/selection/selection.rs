use crate::{editor::selection::region::Region, utils::any::Any};
use derive_more::From;
use nodit::NoditSet;

#[derive(Default, From)]
pub struct Selection {
    regions: NoditSet<usize, Region>,
}

impl Selection {
    pub fn insert(&mut self, region: Region) -> &mut Self {
        self.regions.insert_merge_touching_or_overlapping(region);

        self
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Region> {
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
        // NOTE: curious that NoditSet does not implement FromIterator
        let mut selection = Selection::default();

        for region in iter {
            selection.insert(region);
        }

        selection
    }
}
