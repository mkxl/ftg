use crate::{
    buffer_map::BufferMap, config::Config, tabs_view::TabsView, view::View, view_state::ViewState,
    view_state_set::ViewStateSet,
};
use anyhow::Error as AnyhowError;
use camino::Utf8PathBuf;
use mkutils::{ScrollViewState, ScrollWhen, Utils};
use std::time::Duration;
use ulid::Ulid;

pub struct TabsViewState {
    scroll_view_state: ScrollViewState,
    view_state_set: ViewStateSet,
}

impl TabsViewState {
    const SCROLL_WHEN: ScrollWhen = ScrollWhen::ForLargeContent;
    const RENDER_SCROLL_BARS_TIMEOUT: Option<Duration> = ViewState::RENDER_SCROLL_BARS_TIMEOUT;
    const INITIAL_SCROLL_VIEW_STATE: ScrollViewState =
        ScrollViewState::new(Self::SCROLL_WHEN, Self::RENDER_SCROLL_BARS_TIMEOUT);

    pub fn new() -> Self {
        let scroll_view_state = Self::INITIAL_SCROLL_VIEW_STATE;
        let view_state_set = ViewStateSet::new();

        Self {
            scroll_view_state,
            view_state_set,
        }
    }

    pub fn active_index(&self) -> Option<usize> {
        self.view_state_set.active_index()
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

    pub fn push(&mut self, buffer_id: Ulid, absolute_filepath: Option<Utf8PathBuf>) {
        let view_state = ViewState::new(buffer_id, absolute_filepath);

        self.view_state_set.push(view_state);
    }

    pub fn cycle_next(&mut self) {
        self.view_state_set.cycle_next();
    }

    pub fn cycle_prev(&mut self) {
        self.view_state_set.cycle_prev();
    }

    pub fn get_tabs_view<'a>(&'a mut self, config: &'a Config) -> TabsView<'a> {
        TabsView::new(config, self)
    }
}
