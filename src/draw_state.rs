use crate::{buffer_map::BufferMap, config::Config, view_set::ViewSet};
use anyhow::Error as AnyhowError;
use derive_more::{Constructor, Into};
use mkutils::{Default as MkutilsDefault, Terminal, Utils};
use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
};
use std::borrow::Cow;

#[derive(Constructor, Into, MkutilsDefault)]
struct ViewState<'a> {
    #[default(Self::DEFAULT_TITLE.to_cow_borrowed())]
    title: Cow<'a, str>,
    lines: Vec<Line<'a>>,
}

impl<'a> ViewState<'a> {
    const DEFAULT_TITLE: &'a str = "undefined";
}

struct AreaSet {
    background: Rect,
    buffer_block: Rect,
    buffer_content: Rect,
}

#[derive(Default)]
pub struct DrawState;

impl DrawState {
    const MARGIN_BUFFER_CONTENT: Margin = Margin::new(1, 1);

    fn background(config: &Config) -> Paragraph<'static> {
        Paragraph::default()
            .fg(config.color_scheme.main.fg)
            .bg(config.color_scheme.main.bg)
    }

    fn buffer_block<'a>(config: &Config, buffer_title: &'a str) -> Block<'a> {
        let buffer_border_fg = config
            .color_scheme
            .main
            .bg
            .lighten_fixed(config.theme.buffer_border_lighten);
        let buffer_title_fg = config
            .color_scheme
            .main
            .bg
            .lighten_fixed(config.theme.buffer_title_lighten);
        let buffer_title = buffer_title.fg(buffer_title_fg);

        Block::bordered().fg(buffer_border_fg).title(buffer_title)
    }

    fn buffer_content<'a>(config: &Config, buffer_lines: Vec<Line<'a>>) -> Paragraph<'a> {
        Paragraph::new(buffer_lines).fg(config.color_scheme.main.fg)
    }

    fn view_state<'a>(
        view_set: &'a ViewSet,
        buffer_map: &'a BufferMap,
        frame: &mut Frame,
    ) -> Result<ViewState<'a>, AnyhowError> {
        let Some((active_view, buffer)) = view_set.active_pair(buffer_map)? else {
            return ViewState::default().ok();
        };
        let lines = buffer.lines(active_view.scroll_offset(), frame.area().as_size().into());
        let view_state = ViewState::new(active_view.name(), lines);

        view_state.ok()
    }

    const fn area_set(&self, frame_area: Rect) -> AreaSet {
        let background = frame_area;
        let buffer_block = frame_area;
        let buffer_content = buffer_block.inner(Self::MARGIN_BUFFER_CONTENT);

        AreaSet {
            background,
            buffer_block,
            buffer_content,
        }
    }

    fn draw_impl<'a>(
        &self,
        config: &Config,
        buffer_map: &'a BufferMap,
        view_set: &'a mut ViewSet,
        title: &mut Cow<'a, str>,
        frame: &mut Frame,
    ) -> Result<(), AnyhowError> {
        let area_set = self.area_set(frame.area());

        // NOTE: need this mutable call on [view_set] to occur before i assign
        // [buffer_title] to [title]
        view_set.set_latest_buffer_draw_size(area_set.buffer_content.as_size().into());

        let (buffer_title, buffer_lines) = Self::view_state(view_set, buffer_map, frame)?.into();

        Self::background(config).render_to(frame, area_set.background);
        Self::buffer_block(config, &buffer_title).render_to(frame, area_set.buffer_block);
        Self::buffer_content(config, buffer_lines).render_to(frame, area_set.buffer_content);

        *title = buffer_title;

        ().ok()
    }

    pub fn draw(
        &self,
        config: &Config,
        buffer_map: &BufferMap,
        view_set: &mut ViewSet,
        terminal: &mut Terminal,
    ) -> Result<Vec<u8>, AnyhowError> {
        let mut title = Cow::default();

        terminal
            .draw(|frame| self.draw_impl(config, buffer_map, view_set, &mut title, frame))?
            .set_title(title)?
            .take_byte_str()
            .ok()
    }
}
