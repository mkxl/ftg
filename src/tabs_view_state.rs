use crate::{buffer_map::BufferMap, config::Config, view::View, view_state::ViewState, view_state_set::ViewStateSet};
use anyhow::Error as AnyhowError;
use camino::Utf8PathBuf;
use mkutils::{ScrollViewState, Utils};
use ulid::Ulid;

pub struct TabsViewState {
    scroll_view_state: ScrollViewState,
    view_state_set: ViewStateSet,
}

impl TabsViewState {
    const INITIAL_SCROLL_VIEW_STATE: ScrollViewState = ViewState::INITIAL_SCROLL_VIEW_STATE;

    pub fn new() -> Self {
        let scroll_view_state = Self::INITIAL_SCROLL_VIEW_STATE;
        let view_state_set = ViewStateSet::empty();

        Self {
            scroll_view_state,
            view_state_set,
        }
    }

    pub fn as_slice(&self) -> &[ViewState] {
        self.view_state_set.as_slice()
    }

    pub fn scroll_view_state(&self) -> &ScrollViewState {
        self.scroll_view_state.ref_immut()
    }

    pub fn scroll_view_state_mut(&mut self) -> &mut ScrollViewState {
        self.scroll_view_state.ref_mut()
    }

    pub fn active_view_mut<'a>(
        &'a mut self,
        buffer_map: &'a mut BufferMap,
        config: &'a Config,
    ) -> Result<Option<View<'a>>, AnyhowError> {
        self.view_state_set.active_view_mut(buffer_map, config)
    }

    pub fn push(&mut self, buffer_id: Ulid, filepath: Option<Utf8PathBuf>) {
        let view_state = ViewState::new(buffer_id, filepath);

        self.view_state_set.push(view_state);
    }

    pub fn cycle_next(&mut self) {
        self.view_state_set.cycle_next();
    }

    pub fn cycle_prev(&mut self) {
        self.view_state_set.cycle_prev();
    }
}
