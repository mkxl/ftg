use crate::{buffer::Buffer, buffer_map::BufferMap, region::Region, selection_set::SelectionSet};
use anyhow::Error as AnyhowError;
use camino::Utf8PathBuf;
use mkutils::{
    ActiveVec, Constructor as MkutilsConstructor, PointUsize, ScrollView, ScrollViewState, ScrollWhen, Utils,
};
use ratatui::text::Line as RatatuiLine;
use std::ops::Range;
use ulid::Ulid;

pub struct ViewState {
    buffer_id: Ulid,
    filepath: Option<Utf8PathBuf>,
    selection_set: SelectionSet,
    scroll_view_state: ScrollViewState,
}

impl ViewState {
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
}

#[derive(MkutilsConstructor)]
pub struct View<'a> {
    view_state: &'a mut ViewState,
    buffer: &'a Buffer,
}

impl<'a> View<'a> {
    const DEFAULT_TITLE: &'static str = "untitled";

    pub fn title(&self) -> &str {
        if let Some(filepath) = &self.view_state.filepath {
            filepath.file_name_or_self()
        } else {
            Self::DEFAULT_TITLE
        }
    }
}

impl<'a> ScrollView for View<'a> {
    fn ratatui_lines(&self, rows: Range<usize>, cols: Range<usize>) -> Vec<RatatuiLine<'_>> {
        let scroll_offset = PointUsize::new(cols.start, rows.start);
        let size = PointUsize::new(cols.len(), rows.len());

        self.buffer.raw_ratatui_lines(scroll_offset, size)
    }

    fn scroll_view_state_mut(&mut self) -> &mut ScrollViewState {
        &mut self.view_state.scroll_view_state
    }
}

#[derive(Default)]
pub struct ViewStateSet {
    view_states: Option<ActiveVec<ViewState>>,
}

impl ViewStateSet {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn active_view_mut<'a>(&'a mut self, buffer_map: &'a BufferMap) -> Result<Option<View<'a>>, AnyhowError> {
        let Some(view_states) = &mut self.view_states else { return None.ok() };
        let view_state = view_states.active_mut();
        let buffer = buffer_map.get(&view_state.buffer_id).check_present()?;

        view_state.scroll_view_state.set_latest_content_size(buffer.size());

        View::new(view_state, buffer).some().ok()
    }

    pub fn push(&mut self, view_state: ViewState) {
        if let Some(view_states) = &mut self.view_states {
            let (active_index, _view_state) = view_states.push(view_state);

            view_states.set_active_index(active_index);
        } else {
            self.view_states = ActiveVec::new(view_state).some();
        }
    }
}
