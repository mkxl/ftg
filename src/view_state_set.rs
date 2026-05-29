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

    pub fn push(&mut self, view_state: ViewState) {
        if let Some(view_states) = &mut self.view_states {
            let (active_index, _view_state) = view_states.push(view_state);

            view_states.set_active_index(active_index);
        } else {
            self.view_states = ActiveVec::new(view_state).some();
        }
    }
}
