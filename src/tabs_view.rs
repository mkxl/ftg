use crate::{config::Config, tabs_view_state::TabsViewState, view_state::ViewState};
use crossterm::event::MouseEvent;
use mkutils::{PointI16, PointUsize, ScrollView, ScrollViewState, Utils};
use num::traits::ConstOne;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Styled},
    widgets::Block,
};

pub struct TabsView<'a> {
    config: &'a Config,
    tabs_view_state: &'a mut TabsViewState,
}

impl<'a> TabsView<'a> {
    const TABS_RESIZE_DELTA: PointI16 = PointI16::new(-i16::ONE, -i16::ONE);
    const TAB_HEIGHT: u16 = 4;
    const SCROLL_COUNT: PointUsize = PointUsize::new(usize::ONE, Self::TAB_HEIGHT as usize);

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
        mkutils::max!(
            view_state.title().len_extended_graphemes(),
            view_state.filepath_title().len_extended_graphemes(),
        )
    }

    fn get_content_size(&self) -> PointUsize {
        let view_states = self.tabs_view_state.as_slice();
        let width = view_states.iter().map(Self::view_state_width).max().unwrap_or_default();
        let height = view_states.len().saturating_mul(Self::TAB_HEIGHT.into());

        PointUsize::new(width, height)
    }

    fn set_latest_content_size(&mut self) {
        let content_size = self.get_content_size();

        self.scroll_view_state_mut().set_latest_content_size(content_size);
    }

    pub fn on_mouse_event(&mut self, mouse_event: MouseEvent) {
        self.on_scroll(mouse_event.kind, Self::SCROLL_COUNT);
    }
}

impl ScrollView for TabsView<'_> {
    fn scroll_view_state(&self) -> &ScrollViewState {
        self.tabs_view_state.scroll_view_state()
    }

    fn scroll_view_state_mut(&mut self) -> &mut ScrollViewState {
        self.tabs_view_state.scroll_view_state_mut()
    }

    #[expect(clippy::similar_names)]
    fn render_content(&self, frame: &mut Frame, content_area: Rect) {
        // NOTE: resize so that the tabs don't overlap with the scroll bars
        let tabs_area = content_area.resize_by(&Self::TABS_RESIZE_DELTA);
        let tab_x = tabs_area.x;
        let tab_width = tabs_area.width;
        let scroll_offset = self.scroll_view_state().scroll_offset();
        let tab_offset = scroll_offset.y.saturating_div(Self::TAB_HEIGHT.into());
        let enumerated_view_states = self.tabs_view_state.as_slice().iter().enumerate().skip(tab_offset);

        for (view_state_render_index, (view_state_actual_index, view_state)) in
            enumerated_view_states.enumerated::<u16>()
        {
            let tab_y = Self::TAB_HEIGHT * view_state_render_index;
            let tab_area = Rect::new(tab_x, tab_y, tab_width, Self::TAB_HEIGHT);
            let tab_is_active = view_state_actual_index
                .convert::<usize>()
                .some()
                .eq(&self.tabs_view_state.active_index());
            let tab_color_scheme = self.config.color_scheme.tabs.get(tab_is_active);
            let title_line = view_state
                .title()
                .extended_grapheme_substring(scroll_offset.x..)
                .into_line();
            let filepath_title = view_state.filepath_title();
            let filepath_title_line = filepath_title
                .extended_grapheme_substring(scroll_offset.x..)
                .into_line()
                .set_style(tab_color_scheme.filepath);
            let tab = std::vec![title_line, filepath_title_line]
                .into_paragraph()
                .set_style(tab_color_scheme.tab)
                .block(Block::bordered());

            tab.render_to(frame, tab_area);
        }
    }

    fn scroll_bar_style(&self) -> Style {
        Style::default().fg(self.config.gutter_fg_rgb().into())
    }
}
