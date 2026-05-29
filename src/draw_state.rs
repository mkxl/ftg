use crate::{
    buffer::Buffer,
    buffer_map::BufferMap,
    config::{ColorSpec, Config},
    side_tab_view::SideTabView,
    view::View,
    view_state::ViewState,
    view_state_set::ViewStateSet,
};
use anyhow::Error as AnyhowError;
use mkutils::{Default as MkutilsDefault, ScrollView, ScrollViewState, ScrollWhen, Terminal, Utils};
use num::traits::{ConstOne, ConstZero};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Styled, Stylize},
    text::Text,
    widgets::Paragraph,
};
use std::borrow::Cow;

#[derive(MkutilsDefault)]
pub struct DrawState {
    #[default(Self::DEFAULT_SHOW_STATUS_BAR)]
    show_status_bar: bool,

    #[default(Self::DEFAULT_SHOW_GUTTER)]
    show_gutter: bool,

    #[default(Self::DEFAULT_SHOW_SIDE_TAB)]
    show_side_tab: bool,

    #[default(ScrollViewState::new(Self::SIDE_TAB_SCROLL_WHEN, Self::RENDER_SCROLL_BARS_TIMEOUT.some()))]
    side_tab_scroll_view_state: ScrollViewState,
}

impl DrawState {
    const DEFAULT_SHOW_STATUS_BAR: bool = true;
    const DEFAULT_SHOW_GUTTER: bool = true;
    const DEFAULT_SHOW_SIDE_TAB: bool = true;
    const SIDE_TAB_SCROLL_WHEN: ScrollWhen = ScrollWhen::ForLargeContent;
    const RENDER_SCROLL_BARS_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

    pub const fn show_status_bar_mut(&mut self) -> &mut bool {
        &mut self.show_status_bar
    }

    pub const fn show_gutter_mut(&mut self) -> &mut bool {
        &mut self.show_gutter
    }

    pub const fn show_side_tab_mut(&mut self) -> &mut bool {
        &mut self.show_side_tab
    }

    fn buffer_and_status_bar(&self, frame_area: Rect) -> [Rect; 2] {
        let constraints = [
            Constraint::Fill(u16::ONE),
            Constraint::Length(self.show_status_bar.into()),
        ];
        let layout = Layout::vertical(constraints);

        layout.areas(frame_area)
    }

    fn gutter_line_str(line_num: usize, config: &Config, line_num_width: usize) -> String {
        std::format!(
            "{empty_string:>gutter_padding$}{line_num:>line_num_width$}{empty_string:>gutter_padding$}",
            empty_string = "",
            gutter_padding = config.theme.gutter_padding,
        )
    }

    fn gutter_and_buffer(&self, config: &Config, line_num_width: usize, buffer: Rect) -> [Rect; 2] {
        let gutter_width = if self.show_gutter {
            Self::gutter_line_str(usize::ZERO, config, line_num_width)
                .extended_graphemes()
                .count()
        } else {
            usize::ZERO
        };
        let constraints = [
            Constraint::Length(gutter_width.cast_or_max()),
            Constraint::Fill(u16::ONE),
        ];
        let layout = Layout::horizontal(constraints);

        layout.areas(buffer)
    }

    fn side_tab_and_buffer(&self, view_state_set: &ViewStateSet, buffer: Rect) -> [Rect; 2] {
        let side_tab_width = if self.show_side_tab {
            view_state_set
                .view_states()
                .iter()
                .map(|view_state| view_state.title().extended_graphemes().count())
                .max()
                .unwrap_or(usize::ZERO)
        } else {
            usize::ZERO
        };
        let constraints = [
            Constraint::Length(side_tab_width.cast_or_max()),
            Constraint::Fill(u16::ONE),
        ];
        let layout = Layout::horizontal(constraints);

        layout.areas(buffer)
    }

    fn paragraph(color_spec: ColorSpec) -> Paragraph<'static> {
        Paragraph::default().set_style(color_spec)
    }

    fn background(config: &Config) -> Paragraph<'static> {
        Self::paragraph(config.color_scheme.main)
    }

    fn status_bar(config: &Config) -> Paragraph<'static> {
        Self::paragraph(config.color_scheme.status_bar)
    }

    fn gutter(config: &Config, view: &View, line_num_width: usize) -> Paragraph<'static> {
        view.view_state()
            .scroll_view_state()
            .scroll_offset()
            .y
            .incremented()
            .range_from_len(view.buffer().size().y)
            .map(|line_num| Self::gutter_line_str(line_num, config, line_num_width).fg(config.gutter_fg()))
            .collect::<Text>()
            .pipe_into(Paragraph::new)
    }

    fn draw_impl<'a>(
        &mut self,
        config: &Config,
        buffer_map: &'a mut BufferMap,
        view_state_set: &'a mut ViewStateSet,
        title: &mut Cow<'a, str>,
        frame: &mut Frame,
    ) -> Result<(), AnyhowError> {
        let [buffer, status_bar] = self.buffer_and_status_bar(frame.area());
        let [side_tab, buffer] = self.side_tab_and_buffer(view_state_set, buffer);

        Self::background(config).render_to(frame, frame.area());
        Self::status_bar(config).render_to(frame, status_bar);
        let line_num_width = view_state_set.line_num_width(buffer_map).unwrap_or(usize::ONE);

        if self.show_side_tab {
            let mut side_tab_view = SideTabView::new(view_state_set, &mut self.side_tab_scroll_view_state, config);
            let side_tab_size = side_tab_view.size();

            side_tab_view
                .scroll_view_state_mut()
                .set_latest_content_size(side_tab_size);
            side_tab_view.render(frame, side_tab);
        }

        let mut default_view_state = ViewState::default();
        let mut default_buffer = Buffer::empty();
        let mut view = if let Some(view) = view_state_set.active_view_mut(buffer_map, config)? {
            view
        } else {
            View::new(&mut default_view_state, &mut default_buffer, config)
        };
        let [gutter, buffer] = self.gutter_and_buffer(config, line_num_width, buffer);

        Self::gutter(config, &view, line_num_width).render_to(frame, gutter);
        view.render(frame, buffer);
        title.assign(view.view_state().title().to_owned().into_cow_owned());

        ().ok()
    }

    pub fn draw(
        &mut self,
        config: &Config,
        buffer_map: &mut BufferMap,
        view_state_set: &mut ViewStateSet,
        terminal: &mut Terminal,
    ) -> Result<Vec<u8>, AnyhowError> {
        let mut title = Cow::default();

        terminal
            .draw(|frame| self.draw_impl(config, buffer_map, view_state_set, &mut title, frame))?
            .set_title(title)?
            .take_bytes()
            .ok()
    }
}
