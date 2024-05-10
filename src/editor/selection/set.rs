use crate::{
    editor::selection::{region::Region, selection::Selection},
    utils::any::Any,
};
use derive_more::From;

#[derive(From)]
pub struct SelectionSet {
    selections: Vec<Selection>,
}

impl SelectionSet {
    pub fn primary(&self) -> &Selection {
        &self.selections[0]
    }
}

impl From<Region> for SelectionSet {
    fn from(region: Region) -> Self {
        Selection::from(region).into()
    }
}

impl From<Selection> for SelectionSet {
    fn from(selection: Selection) -> Self {
        selection.once().collect()
    }
}

impl FromIterator<Region> for SelectionSet {
    fn from_iter<T: IntoIterator<Item = Region>>(iter: T) -> Self {
        iter.into_iter().collect::<Selection>().into()
    }
}

impl FromIterator<Selection> for SelectionSet {
    fn from_iter<T: IntoIterator<Item = Selection>>(iter: T) -> Self {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}
