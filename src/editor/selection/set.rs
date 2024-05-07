use crate::{editor::selection::selection::Selection, utils::any::Any};

pub struct SelectionSet {
    selections: Vec<Selection>,
}

impl Default for SelectionSet {
    fn default() -> Self {
        Selection::empty().into()
    }
}

impl From<Selection> for SelectionSet {
    fn from(selection: Selection) -> Self {
        selection.once().collect()
    }
}

impl FromIterator<Selection> for SelectionSet {
    fn from_iter<T: IntoIterator<Item = Selection>>(iter: T) -> Self {
        let selections = iter.into_iter().collect();

        Self { selections }
    }
}
