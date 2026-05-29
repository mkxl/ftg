use crate::{config::Config, view_state::ViewState, view_state_set::ViewStateSet};
use derive_more::Constructor;
use mkutils::{Orientation, PointUsize, ScrollView, ScrollViewState, Utils};
use num::traits::{ConstOne, ConstZero};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::Line as RatatuiLine,
};
use std::borrow::Cow;

#[derive(Constructor)]
pub struct SideTabView<'a> {
    view_state_set: &'a ViewStateSet,
    scroll_view_state: &'a mut ScrollViewState,
    config: &'a Config,
}

impl SideTabView<'_> {
    const CARD_HEIGHT: usize = 4;
    const HORIZONTAL_PADDING: usize = 1;
    const BORDER_WIDTH: usize = 2;

    pub fn width(view_state_set: &ViewStateSet) -> usize {
        view_state_set
            .view_states()
            .iter()
            .map(|view_state| {
                Self::tab_file_name(view_state)
                    .extended_graphemes()
                    .count()
                    .max(Self::tab_filepath(view_state).extended_graphemes().count())
            })
            .max()
            .unwrap_or(usize::ZERO)
            .max(usize::ONE)
            .saturating_add(Self::HORIZONTAL_PADDING * 2)
            .saturating_add(Self::BORDER_WIDTH)
    }

    pub fn size(&self) -> PointUsize {
        let width = Self::width(self.view_state_set);
        let height = self
            .view_state_set
            .view_states()
            .len()
            .saturating_mul(Self::CARD_HEIGHT);

        PointUsize::new(width, height)
    }

    fn tab_file_name(view_state: &ViewState) -> &str {
        view_state.title()
    }

    fn tab_filepath(view_state: &ViewState) -> Cow<'_, str> {
        view_state.filepath().map_or_else(
            || view_state.title().to_cow_borrowed(),
            |filepath| filepath.as_str().unexpand_user().to_string().into_cow_owned(),
        )
    }

    fn visible_string(string: &str, cols: std::ops::Range<usize>) -> String {
        string
            .extended_graphemes()
            .skip(cols.start)
            .take(cols.len_range())
            .collect()
    }

    fn padded_string(string: &str, width: usize) -> String {
        let padding_width = width.saturating_sub(string.extended_graphemes().count());

        string.to_owned() + &" ".repeat(padding_width)
    }

    fn card_line(
        &self,
        view_state: &ViewState,
        index: usize,
        card_row: usize,
        cols: std::ops::Range<usize>,
    ) -> RatatuiLine<'static> {
        let modifier = (index == self.view_state_set.active_index()).if_else(Modifier::REVERSED, Modifier::empty());
        let content_width = Self::width(self.view_state_set).saturating_sub(Self::BORDER_WIDTH);
        let text_width = content_width.saturating_sub(Self::HORIZONTAL_PADDING * 2);
        let tab_filepath = Self::tab_filepath(view_state);
        let string = match card_row {
            0 | 3 => "+".to_owned() + &"-".repeat(content_width) + "+",
            1 => "| ".to_owned() + &Self::padded_string(Self::tab_file_name(view_state), text_width) + " |",
            2 => "| ".to_owned() + &Self::padded_string(&tab_filepath, text_width) + " |",
            _ => String::new(),
        };

        modifier.stylize(Self::visible_string(&string, cols)).into()
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

        for row in rows {
            let Some(row_area) = row_areas.next() else { break };
            let index = row / Self::CARD_HEIGHT;
            let card_row = row % Self::CARD_HEIGHT;
            let Some(view_state) = self.view_state_set.view_states().get(index) else { break };

            self.card_line(view_state, index, card_row, cols.clone())
                .render_to(frame, row_area);
        }
    }

    fn scroll_bar_style(&self) -> Style {
        Style::default().fg(self.config.gutter_fg().into())
    }

    fn render_scroll_bars(&self, frame: &mut Frame, content_area: Rect) {
        let scroll_view_state = self.scroll_view_state();

        if !scroll_view_state.should_render_scroll_bars() || content_area.height < 2 || content_area.width == 0 {
            return;
        }

        if scroll_view_state.max_scroll_offset().y.is_positive() {
            scroll_view_state
                .scroll_bar(Orientation::Vertical, self.scroll_bar_style())
                .render(content_area, frame.buffer_mut());
        }
    }
}
