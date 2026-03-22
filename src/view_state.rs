use crate::{region::Region, selection::Selection, selection_set::SelectionSet};
use camino::Utf8PathBuf;
use mkutils::{ScrollViewState, ScrollWhen, Utils};
use ulid::Ulid;

pub struct ViewState {
    buffer_id: Ulid,
    filepath: Option<Utf8PathBuf>,
    selection_set: SelectionSet,
    scroll_view_state: ScrollViewState,
}

impl ViewState {
    const DEFAULT_TITLE: &'static str = "untitled";
    const SCROLL_WHEN: ScrollWhen = ScrollWhen::ForLargeContent;

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

    pub const fn buffer_id(&self) -> Ulid {
        self.buffer_id
    }

    pub const fn scroll_view_state(&self) -> &ScrollViewState {
        &self.scroll_view_state
    }

    pub const fn scroll_view_state_mut(&mut self) -> &mut ScrollViewState {
        &mut self.scroll_view_state
    }

    pub const fn selection_set(&self) -> &SelectionSet {
        &self.selection_set
    }

    pub const fn selection_set_mut(&mut self) -> &mut SelectionSet {
        &mut self.selection_set
    }

    pub fn title(&self) -> &str {
        if let Some(filepath) = &self.filepath {
            filepath.file_name_or_self()
        } else {
            Self::DEFAULT_TITLE
        }
    }

    pub fn set_active_selection(&mut self, new_active_selection: Selection) {
        self.selection_set
            .active_vec_mut()
            .active_mut()
            .assign(new_active_selection);
    }
}

impl Default for ViewState {
    fn default() -> Self {
        Self::new(Ulid::default(), None)
    }
}
