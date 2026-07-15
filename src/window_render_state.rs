use crate::{
    buffer::Buffer,
    buffer_map::BufferMap,
    config::{ColorSpec, Config},
    tabs_view_state::TabsViewState,
    view::View,
    view_state::ViewState,
};
use anyhow::Error as AnyhowError;
use mkutils::{Default as MkutilsDefault, ScrollView, Terminal, Utils};
use num::traits::{ConstOne, ConstZero};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Rect},
    style::{Styled, Stylize},
    text::Text,
    widgets::{Clear, Paragraph},
};
use std::borrow::Cow;

#[derive(Default)]
pub struct WindowAreaSet {
    pub background: Rect,
    pub tabs: Rect,
    pub view: Rect,
    pub status_bar: Rect,
}

struct ViewAreaSet {
    gutter: Rect,
    buffer: Rect,
}

#[derive(MkutilsDefault)]
pub struct WindowRenderState {
    #[default(Self::DEFAULT_SHOW_STATUS_BAR)]
    show_status_bar: bool,

    #[default(Self::DEFAULT_SHOW_GUTTER)]
    show_gutter: bool,

    #[default(Self::DEFAULT_SHOW_TABS)]
    show_tabs: bool,

    latest_window_area_set: WindowAreaSet,
}

impl WindowRenderState {
    const DEFAULT_SHOW_GUTTER: bool = true;
    const DEFAULT_SHOW_STATUS_BAR: bool = true;
    const DEFAULT_SHOW_TABS: bool = true;
    const MAX_TABS_WIDTH_PERCENTAGE: u16 = 100;

    pub const fn latest_window_area_set(&self) -> &WindowAreaSet {
        &self.latest_window_area_set
    }

    pub fn toggle_status_bar(&mut self) {
        self.show_status_bar.toggle();
    }

    pub fn toggle_gutter(&mut self) {
        self.show_gutter.toggle();
    }

    pub fn toggle_tabs(&mut self) {
        self.show_tabs.toggle();
    }

    const fn get_tabs_width(&self, config: &Config, non_status_bar_area: Rect) -> u16 {
        if self.show_tabs {
            non_status_bar_area
                .width
                .saturating_mul(config.theme.tabs_width_percentage)
                .saturating_div(Self::MAX_TABS_WIDTH_PERCENTAGE)
        } else {
            u16::ZERO
        }
    }

    fn get_window_area_set(&self, config: &Config, background: Rect) -> WindowAreaSet {
        let [non_status_bar_area, status_bar] = [
            Constraint::Fill(u16::ONE),
            Constraint::Length(self.show_status_bar.into()),
        ]
        .layout_areas(Direction::Vertical, background);
        let tabs_width = self.get_tabs_width(config, non_status_bar_area);
        let [tabs, view] = [Constraint::Length(tabs_width), Constraint::Fill(u16::ONE)]
            .layout_areas(Direction::Horizontal, non_status_bar_area);

        WindowAreaSet {
            background,
            tabs,
            view,
            status_bar,
        }
    }

    fn gutter_line_str(line_num: usize, config: &Config, line_num_width: usize) -> String {
        std::format!(
            "{empty_string:>gutter_padding$}{line_num:>line_num_width$}{empty_string:>gutter_padding$}",
            empty_string = "",
            gutter_padding = config.theme.gutter_padding,
        )
    }

    fn get_view_area_set(&self, config: &Config, view: &View, view_area: Rect) -> ViewAreaSet {
        let gutter_width = if self.show_gutter {
            Self::gutter_line_str(usize::ZERO, config, view.line_num_width())
                .extended_graphemes()
                .count()
        } else {
            usize::ZERO
        };
        let [gutter, buffer] = [
            Constraint::Length(gutter_width.cast_or_max()),
            Constraint::Fill(u16::ONE),
        ]
        .layout_areas(Direction::Horizontal, view_area);

        ViewAreaSet { gutter, buffer }
    }

    fn render_color_spec(color_spec: ColorSpec, frame: &mut Frame, area: Rect) {
        Clear.render_to(frame, area);
        Paragraph::default().set_style(color_spec).render_to(frame, area);
    }

    fn get_gutter(config: &Config, view: &View) -> Paragraph<'static> {
        let line_num_width = view.line_num_width();
        let gutter_fg_rgb = config.gutter_fg_rgb();

        view.view_state()
            .scroll_view_state()
            .scroll_offset()
            .y
            .incremented()
            .range_from_len(view.buffer().size().y)
            .map(|line_num| Self::gutter_line_str(line_num, config, line_num_width).fg(gutter_fg_rgb))
            .collect::<Text>()
            .into_paragraph()
    }

    fn draw_window(&self, config: &Config, tabs_view_state: &mut TabsViewState, frame: &mut Frame) {
        Self::render_color_spec(config.color_scheme.main, frame, self.latest_window_area_set.background);
        tabs_view_state
            .get_tabs_view(config)
            .render(frame, self.latest_window_area_set.tabs);
        Self::render_color_spec(
            config.color_scheme.status_bar,
            frame,
            self.latest_window_area_set.status_bar,
        );
    }

    fn draw_view<'a>(
        &self,
        config: &'a Config,
        buffer_map: &'a mut BufferMap,
        tabs_view_state: &'a mut TabsViewState,
        default_view: View<'a>,
        title: &mut Cow<'a, str>,
        frame: &mut Frame,
    ) -> Result<(), AnyhowError> {
        let mut view = tabs_view_state
            .active_view_mut(buffer_map, config)?
            .unwrap_or(default_view);
        let view_area_set = self.get_view_area_set(config, &view, self.latest_window_area_set.view);

        Self::get_gutter(config, &view).render_to(frame, view_area_set.gutter);
        view.render(frame, view_area_set.buffer);
        title.assign(view.into_view_state().title().to_cow_borrowed());

        ().ok()
    }

    fn draw_impl<'a>(
        &mut self,
        config: &'a Config,
        buffer_map: &'a mut BufferMap,
        tabs_view_state: &'a mut TabsViewState,
        default_view: View<'a>,
        title: &mut Cow<'a, str>,
        frame: &mut Frame,
    ) -> Result<(), AnyhowError> {
        self.latest_window_area_set = self.get_window_area_set(config, frame.area());

        self.draw_window(config, tabs_view_state, frame);
        self.draw_view(config, buffer_map, tabs_view_state, default_view, title, frame)?;

        ().ok()
    }

    pub fn draw(
        &mut self,
        config: &Config,
        buffer_map: &mut BufferMap,
        tabs_view_state: &mut TabsViewState,
        terminal: &mut Terminal,
    ) -> Result<Vec<u8>, AnyhowError> {
        let mut default_view_state = ViewState::default();
        let mut default_buffer = Buffer::new();
        let default_view = View::new(default_view_state.ref_mut(), default_buffer.ref_mut(), config);
        let mut title = Cow::<str>::default();

        terminal
            .draw(|frame| {
                self.draw_impl(
                    config,
                    buffer_map,
                    tabs_view_state,
                    default_view,
                    title.ref_mut(),
                    frame,
                )
            })?
            .set_title(title)?
            .take_bytes()
            .ok()
    }
}
