use crate::{region::Region, selection::Selection};
use camino::Utf8PathBuf;
use mkutils::{ActiveVec, ScrollViewState, ScrollWhen, Utils};
use std::{borrow::Cow, time::Duration};
use ulid::Ulid;

pub struct ViewState {
    buffer_id: Ulid,
    absolute_filepath: Option<Utf8PathBuf>,
    selections: ActiveVec<Selection>,
    scroll_view_state: ScrollViewState,
}

impl ViewState {
    pub const DEFAULT_TITLE: &'static str = "untitled";
    pub const RENDER_SCROLL_BARS_TIMEOUT: Option<Duration> = Some(Duration::from_secs(1));

    const SCROLL_WHEN: ScrollWhen = ScrollWhen::ForLargeContent;
    const INITIAL_SCROLL_VIEW_STATE: ScrollViewState =
        ScrollViewState::new(Self::SCROLL_WHEN, Self::RENDER_SCROLL_BARS_TIMEOUT);

    pub fn new(buffer_id: Ulid, absolute_filepath: Option<Utf8PathBuf>) -> Self {
        let selections = Self::initial_selections();
        let scroll_view_state = Self::INITIAL_SCROLL_VIEW_STATE;

        Self {
            buffer_id,
            absolute_filepath,
            selections,
            scroll_view_state,
        }
    }

    fn initial_selections() -> ActiveVec<Selection> {
        ActiveVec::new(Region::ZERO.into())
    }

    pub const fn buffer_id(&self) -> Ulid {
        self.buffer_id
    }

    pub const fn scroll_view_state(&self) -> &ScrollViewState {
        &self.scroll_view_state
    }

    pub const fn scroll_view_state_mut(&mut self) -> &mut ScrollViewState {
        &mut self.scroll_view_state
    }

    pub const fn selections(&self) -> &ActiveVec<Selection> {
        &self.selections
    }

    pub const fn selections_mut(&mut self) -> &mut ActiveVec<Selection> {
        &mut self.selections
    }

    pub fn title(&self) -> &str {
        self.absolute_filepath
            .as_ref()
            .map_or(Self::DEFAULT_TITLE, Utf8PathBuf::file_name_or_self)
    }

    pub fn filepath_title(&self) -> Cow<'_, str> {
        if let Some(absolute_filepath) = self.absolute_filepath.as_deref() {
            absolute_filepath.unexpand_user().into_cow()
        } else {
            Self::DEFAULT_TITLE.to_cow_borrowed()
        }
    }

    pub fn set_active_selection(&mut self, new_active_selection: Selection) {
        self.selections.active_mut().assign(new_active_selection);
    }
}

impl Default for ViewState {
    fn default() -> Self {
        Self::new(Ulid::default(), None)
    }
}
