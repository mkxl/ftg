use crate::{region::Region, selection_set::SelectionSet};
use camino::Utf8PathBuf;
use mkutils::{PointUsize, ScrollViewState, ScrollWhen, Utils};
use std::borrow::Cow;
use ulid::Ulid;

pub struct View {
    buffer_id: Ulid,
    filepath: Option<Utf8PathBuf>,
    selection_set: SelectionSet,
    scroll_view_state: ScrollViewState,
}

impl View {
    const SCROLL_WHEN: ScrollWhen = ScrollWhen::ForLargeContent;
    const DEFAULT_NAME: &str = "untitled";

    pub fn new(buffer_id: Ulid, filepath: Option<Utf8PathBuf>) -> Self {
        let selection_set = Self::initial_selection_set();
        let scroll_view_state = ScrollViewState::new(Self::SCROLL_WHEN);

        Self {
            buffer_id,
            filepath,
            selection_set,
            scroll_view_state,
        }
    }

    fn initial_selection_set() -> SelectionSet {
        Region::ZERO.into()
    }

    pub fn buffer_id(&self) -> Ulid {
        self.buffer_id
    }

    pub fn name(&self) -> Cow<'_, str> {
        let Some(filepath) = &self.filepath else { return Self::DEFAULT_NAME.to_cow_borrowed() };

        match filepath.unexpand_user() {
            Cow::Borrowed(filepath) => filepath.as_str().to_cow_borrowed(),
            Cow::Owned(filepath) => filepath.into_string().into_cow_owned(),
        }
    }

    pub const fn scroll_view_state(&self) -> &ScrollViewState {
        &self.scroll_view_state
    }

    pub const fn scroll_view_state_mut(&mut self) -> &mut ScrollViewState {
        &mut self.scroll_view_state
    }
}
