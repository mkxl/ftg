use crate::{config::Config, view_state::ViewState, view_state_set::ViewStateSet};
use derive_more::Constructor;
use mkutils::{Orientation, PointUsize, ScrollView, ScrollViewState, Utils};
use num::traits::{ConstOne, ConstZero};
use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Style, Styled},
    text::{Line as RatatuiLine, Text},
    widgets::{Block, Paragraph},
};
use std::borrow::Cow;

#[derive(Constructor)]
pub struct SideTabView<'a> {
    view_state_set: &'a ViewStateSet,
    scroll_view_state: &'a mut ScrollViewState,
    config: &'a Config,
}

impl SideTabView<'_> {
    pub const CARD_HEIGHT: usize = 4;
    const HORIZONTAL_PADDING: usize = 1;
    const CONTENT_AREA_WIDTH_MARGIN: u16 = 1;
    const CARD_CONTENT_MARGIN: Margin = Margin::new(1, 1);

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
            .saturating_add(Self::CARD_CONTENT_MARGIN.horizontal.cast_or_max())
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

    fn side_tab_content_size(&self, content_area: Rect) -> PointUsize {
        let width = Self::width(self.view_state_set);
        let height = self
            .view_state_set
            .view_states()
            .len()
            .saturating_sub(1)
            .saturating_mul(Self::CARD_HEIGHT)
            .saturating_add(content_area.height.into());

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

    fn block(&self, index: usize) -> Block<'static> {
        let border_style = Style::from(self.config.color_scheme.side_tab);
        let style = (index == self.view_state_set.active_index())
            .if_else(Style::from(self.config.color_scheme.side_tab_active), border_style);

        Block::bordered().border_style(border_style).style(style)
    }

    fn text(
        &self,
        view_state: &ViewState,
        index: usize,
        cols: std::ops::Range<usize>,
        text_width: usize,
    ) -> Text<'static> {
        let (file_name_style, filepath_style) = if index == self.view_state_set.active_index() {
            (
                self.config.color_scheme.side_tab_active,
                self.config.color_scheme.side_tab_path_active,
            )
        } else {
            (
                self.config.color_scheme.side_tab,
                self.config.color_scheme.side_tab_path,
            )
        };
        let file_name = Self::visible_string(Self::tab_file_name(view_state), cols.clone());
        let file_name = Self::padded_string(&file_name, text_width).set_style(file_name_style);
        let tab_filepath = Self::tab_filepath(view_state);
        let filepath = Self::visible_string(&tab_filepath, cols);
        let filepath = Self::padded_string(&filepath, text_width).set_style(filepath_style);

        [RatatuiLine::from(file_name), RatatuiLine::from(filepath)]
            .into_iter()
            .collect()
    }

    fn render_card(
        &self,
        frame: &mut Frame,
        view_state: &ViewState,
        index: usize,
        card_area: Rect,
        cols: std::ops::Range<usize>,
    ) {
        let block = self.block(index);
        let text_area = card_area.inner(Self::CARD_CONTENT_MARGIN);
        let text_width = text_area.width.into();

        block.render_to(frame, card_area);
        Paragraph::new(self.text(view_state, index, cols, text_width)).render_to(frame, text_area);
    }
}

impl ScrollView for SideTabView<'_> {
    fn scroll_view_state(&self) -> &ScrollViewState {
        self.scroll_view_state
    }

    fn scroll_view_state_mut(&mut self) -> &mut ScrollViewState {
        self.scroll_view_state
    }

    fn content_area(&self, scroll_view_area: Rect) -> Rect {
        Rect {
            width: scroll_view_area.width.saturating_sub(Self::CONTENT_AREA_WIDTH_MARGIN),
            ..scroll_view_area
        }
    }

    fn render(&mut self, frame: &mut Frame, scroll_view_area: Rect) {
        let content_area = self.content_area(scroll_view_area);
        let content_size = self.side_tab_content_size(content_area);

        self.scroll_view_state_mut().set_latest_content_size(content_size);
        self.scroll_view_state_mut()
            .set_latest_scroll_view_area_size(content_area.as_size().into());
        self.render_content(frame, content_area);
        self.render_scroll_bars(frame, scroll_view_area);
        self.render_misc(frame, scroll_view_area);
    }

    fn render_content(&self, frame: &mut Frame, content_area: Rect) {
        let scroll_offset = self.scroll_view_state().scroll_offset();
        let rows = scroll_offset.y.range_from_len(content_area.height.into());
        let cols = scroll_offset.x.range_from_len(content_area.width.into());
        let card_height = Self::CARD_HEIGHT.cast_or_max::<u16>();

        for index in 0..self.view_state_set.view_states().len() {
            let card_y = index.saturating_mul(Self::CARD_HEIGHT);
            let Some(card_screen_y) = card_y.checked_sub(scroll_offset.y) else { continue };
            let card_bottom = card_screen_y.saturating_add(Self::CARD_HEIGHT);

            if rows.end <= card_screen_y {
                break;
            }

            if card_bottom <= rows.start {
                continue;
            }

            let Some(view_state) = self.view_state_set.view_states().get(index) else { break };
            let card_area_y = content_area.y + card_screen_y.cast_or_max::<u16>();
            let card_area_height = card_height.min(content_area.bottom().saturating_sub(card_area_y));
            let card_area = Rect::new(content_area.x, card_area_y, content_area.width, card_area_height);

            self.render_card(frame, view_state, index, card_area, cols.clone());
        }
    }

    fn scroll_bar_style(&self) -> Style {
        Style::default().fg(self.config.gutter_fg().into())
    }

    fn render_scroll_bars(&self, frame: &mut Frame, scroll_view_area: Rect) {
        let scroll_view_state = self.scroll_view_state();

        if !scroll_view_state.should_render_scroll_bars() || scroll_view_area.height < 2 || scroll_view_area.width == 0
        {
            return;
        }

        if scroll_view_state.max_scroll_offset().y.is_positive() {
            scroll_view_state
                .scroll_bar(Orientation::Vertical, self.scroll_bar_style())
                .render(scroll_view_area, frame.buffer_mut());
        }
    }
}
