use crate::{config::Config, view_state::ViewState, view_state_set::ViewStateSet};
use derive_more::Constructor;
use mkutils::{PointUsize, ScrollView, ScrollViewState, Utils};
use num::traits::{ConstOne, ConstZero};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::Line as RatatuiLine,
};

#[derive(Constructor)]
pub struct SideTabView<'a> {
    view_state_set: &'a ViewStateSet,
    scroll_view_state: &'a mut ScrollViewState,
    config: &'a Config,
}

impl SideTabView<'_> {
    pub fn size(&self) -> PointUsize {
        let width = self
            .view_state_set
            .view_states()
            .iter()
            .map(|view_state| view_state.title().extended_graphemes().count())
            .max()
            .unwrap_or(usize::ZERO)
            .max(usize::ONE);

        PointUsize::new(width, self.view_state_set.view_states().len())
    }

    fn render_title<'a>(
        &self,
        view_state: &'a ViewState,
        index: usize,
        cols: std::ops::Range<usize>,
    ) -> RatatuiLine<'a> {
        let modifier = (index == self.view_state_set.active_index()).if_else(Modifier::REVERSED, Modifier::empty());

        view_state
            .title()
            .extended_graphemes()
            .skip(cols.start)
            .take(cols.len_range())
            .map(|extended_grapheme| modifier.stylize(extended_grapheme))
            .collect()
    }
}

impl ScrollView for SideTabView<'_> {
    fn scroll_view_state(&self) -> &ScrollViewState {
        self.scroll_view_state
    }

    fn scroll_view_state_mut(&mut self) -> &mut ScrollViewState {
        self.scroll_view_state
    }

    fn render_content(&self, frame: &mut Frame, content_area: Rect) {
        let scroll_offset = self.scroll_view_state().scroll_offset();
        let rows = scroll_offset.y.range_from_len(content_area.height.into());
        let cols = scroll_offset.x.range_from_len(content_area.width.into());
        let mut row_areas = content_area.rows();

        for (index, view_state) in self
            .view_state_set
            .view_states()
            .iter()
            .enumerate()
            .skip(rows.start)
            .take(rows.len())
        {
            let Some(row_area) = row_areas.next() else { break };

            self.render_title(view_state, index, cols.clone())
                .render_to(frame, row_area);
        }
    }

    fn scroll_bar_style(&self) -> Style {
        Style::default().fg(self.config.gutter_fg().into())
    }
}
