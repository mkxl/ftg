use crate::selection::Selection;
use derive_more::From;
use mkutils::ActiveVec;

#[derive(From)]
pub struct SelectionSet {
    selections: ActiveVec<Selection>,
}

impl SelectionSet {
    pub fn new(selection: Selection) -> Self {
        ActiveVec::new(selection).into()
    }
}

impl From<Selection> for SelectionSet {
    fn from(selection: Selection) -> Self {
        Self::new(selection)
    }
}
