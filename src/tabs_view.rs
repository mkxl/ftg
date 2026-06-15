use crate::{config::Config, tabs_view_state::TabsViewState, view_state::ViewState};
use mkutils::{PointUsize, ScrollView, ScrollViewState, Utils};
use ratatui::{Frame, layout::Rect, style::Style};

pub struct TabsView<'a> {
    config: &'a Config,
    tabs_view_state: &'a mut TabsViewState,
}

impl<'a> TabsView<'a> {
    pub fn new(config: &'a Config, tabs_view_state: &'a mut TabsViewState) -> Self {
        let mut tabs_view = Self {
            config,
            tabs_view_state,
        };

        // NOTE-7d8ab9
        tabs_view.set_latest_content_size();

        tabs_view
    }

    fn view_state_width(view_state: &ViewState) -> usize {
        view_state.title().extended_graphemes().count()
    }

    fn get_content_size(&self) -> PointUsize {
        let view_states = self.tabs_view_state.as_slice();
        let width = view_states.iter().map(Self::view_state_width).max().unwrap_or_default();

        PointUsize::new(width, view_states.len())
    }

    fn set_latest_content_size(&mut self) {
        let content_size = self.get_content_size();

        self.scroll_view_state_mut().set_latest_content_size(content_size);
    }
}

impl ScrollView for TabsView<'_> {
    fn scroll_view_state(&self) -> &ScrollViewState {
        self.tabs_view_state.scroll_view_state()
    }

    fn scroll_view_state_mut(&mut self) -> &mut ScrollViewState {
        self.tabs_view_state.scroll_view_state_mut()
    }

    fn render_content(&self, _frame: &mut Frame, _content_area: Rect) {}

    fn scroll_bar_style(&self) -> Style {
        Style::default().fg(self.config.gutter_fg_rgb().into())
    }
}
