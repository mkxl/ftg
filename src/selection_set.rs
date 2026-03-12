use crate::{region::Region, selection::Selection};
use derive_more::From;
use mkutils::{ActiveVec, FromChain};

#[derive(From, FromChain)]
#[from_chain(Region, Selection, ActiveVec<Selection>)]
#[from_chain(Selection, ActiveVec<Selection>)]
pub struct SelectionSet {
    selections: ActiveVec<Selection>,
}

impl SelectionSet {
    pub fn new(selection: Selection) -> Self {
        ActiveVec::new(selection).into()
    }

    pub const fn active_vec(&self) -> &ActiveVec<Selection> {
        &self.selections
    }

    pub const fn active_vec_mut(&mut self) -> &mut ActiveVec<Selection> {
        &mut self.selections
    }
}
