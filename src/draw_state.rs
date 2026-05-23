use crate::{
    buffer::Buffer,
    buffer_map::BufferMap,
    config::{ColorSpec, Config},
    view::View,
    view_state::ViewState,
    view_state_set::ViewStateSet,
};
use anyhow::Error as AnyhowError;
use mkutils::{Default as MkutilsDefault, ScrollView, Terminal, Utils};
use num::traits::{ConstOne, ConstZero};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Styled, Stylize},
    text::Text,
    widgets::Paragraph,
};
use std::borrow::Cow;

struct AreaSet {
    background: Rect,
    gutter: Rect,
    buffer: Rect,
    status_bar: Rect,
}

#[derive(MkutilsDefault)]
pub struct DrawState {
    #[default(Self::DEFAULT_SHOW_STATUS_BAR)]
    show_status_bar: bool,

    #[default(Self::DEFAULT_SHOW_GUTTER)]
    show_gutter: bool,
}

impl DrawState {
    const EMPTY_VIEWS_TITLE: &str = "undefined";
    const DEFAULT_SHOW_STATUS_BAR: bool = true;
    const DEFAULT_SHOW_GUTTER: bool = true;

    pub const fn show_status_bar_mut(&mut self) -> &mut bool {
        &mut self.show_status_bar
    }

    pub const fn show_gutter_mut(&mut self) -> &mut bool {
        &mut self.show_gutter
    }

    fn area_set(&self, config: &Config, view: &View, frame_area: Rect) -> AreaSet {
        let background = frame_area;
        let [buffer, status_bar] = self.buffer_and_status_bar(frame_area);
        let [gutter, buffer] = self.gutter_and_buffer(config, view, buffer);

        AreaSet {
            background,
            gutter,
            buffer,
            status_bar,
        }
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

    fn gutter_and_buffer(&self, config: &Config, view: &View, buffer: Rect) -> [Rect; 2] {
        let gutter_width = if self.show_gutter {
            Self::gutter_line_str(usize::ZERO, config, view.line_num_width())
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

    fn paragraph(color_spec: ColorSpec) -> Paragraph<'static> {
        Paragraph::default().set_style(color_spec)
    }

    fn background(config: &Config) -> Paragraph<'static> {
        Self::paragraph(config.color_scheme.main)
    }

    fn status_bar(config: &Config) -> Paragraph<'static> {
        Self::paragraph(config.color_scheme.status_bar)
    }

    fn gutter(config: &Config, view: &View) -> Paragraph<'static> {
        let line_num_width = view.line_num_width();

        view.view_state()
            .scroll_view_state()
            .scroll_offset()
            .y
            .incremented()
            .range_from_len(view.buffer().rope().size().y)
            .map(|line_num| Self::gutter_line_str(line_num, config, line_num_width).fg(config.gutter_fg()))
            .collect::<Text>()
            .pipe_into(Paragraph::new)
    }

    fn draw_impl<'a>(
        &self,
        config: &Config,
        buffer_map: &'a mut BufferMap,
        view_state_set: &'a mut ViewStateSet,
        title: &mut Cow<'a, str>,
        frame: &mut Frame,
    ) -> Result<(), AnyhowError> {
        let mut default_view_state = ViewState::default();
        let mut default_buffer = Buffer::empty();
        let mut view = if let Some(view) = view_state_set.active_view_mut(buffer_map, config)? {
            view
        } else {
            View::new(&mut default_view_state, &mut default_buffer, config)
        };
        let area_set = self.area_set(config, &view, frame.area());

        Self::background(config).render_to(frame, area_set.background);
        Self::status_bar(config).render_to(frame, area_set.status_bar);
        Self::gutter(config, &view).render_to(frame, area_set.gutter);
        view.render(frame, area_set.buffer);
        title.assign(view.view_state().title().to_owned().into_cow_owned());

        ().ok()
    }

    pub fn draw(
        &self,
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
