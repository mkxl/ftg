use crate::{buffer_map::BufferMap, config::Config, view::View, view_state::ViewState};
use anyhow::Error as AnyhowError;
use mkutils::{ActiveVec, Utils};

#[derive(Default)]
pub struct ViewStateSet {
    view_states: Option<ActiveVec<ViewState>>,
}

impl ViewStateSet {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn active_view_mut<'a>(
        &'a mut self,
        buffer_map: &'a mut BufferMap,
        config: &'a Config,
    ) -> Result<Option<View<'a>>, AnyhowError> {
        let Some(view_states) = &mut self.view_states else { return None.ok() };
        let view_state = view_states.active_mut();
        let buffer = buffer_map.get_mut(&view_state.buffer_id()).check_present()?;

        // NOTE: see [latest-content-size-abb4a0]
        view_state
            .scroll_view_state_mut()
            .set_latest_content_size(buffer.size());

        View::new(view_state, buffer, config).some().ok()
    }

    pub fn view_states(&self) -> &[ViewState] {
        self.view_states.as_ref().map_or([].as_slice(), ActiveVec::as_slice)
    }

    pub fn active_index(&self) -> usize {
        self.view_states.as_ref().map_or(0, ActiveVec::active_index)
    }

    pub fn line_num_width(&self, buffer_map: &BufferMap) -> Option<usize> {
        self.view_states()
            .iter()
            .filter_map(|view_state| buffer_map.get(&view_state.buffer_id()))
            .map(|buffer| buffer.size().y.ilog10().incremented().cast_or_max())
            .max()
    }

    pub fn cycle_prev(&mut self) {
        self.view_states.as_mut().map(ActiveVec::cycle_prev);
    }

    pub fn cycle_next(&mut self) {
        self.view_states.as_mut().map(ActiveVec::cycle_next);
    }

    pub fn push(&mut self, view_state: ViewState) {
        if let Some(view_states) = &mut self.view_states {
            view_states.push(view_state);
        } else {
            self.view_states = ActiveVec::new(view_state).some();
        }
    }
}
