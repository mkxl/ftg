use crate::{buffer_map::BufferMap, config::Config, view_set::ViewSet};
use anyhow::Error as AnyhowError;
use derive_more::{Constructor, Into};
use mkutils::{Default as MkutilsDefault, Terminal, Timer, Utils};
use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::Stylize,
    text::Line as RatatuiLine,
    widgets::{Block, Paragraph},
};
use std::borrow::Cow;

#[derive(Constructor, Into, MkutilsDefault)]
struct ViewState<'a> {
    #[default(Self::DEFAULT_TITLE.to_cow_borrowed())]
    title: Cow<'a, str>,
    raw_ratatui_lines: Vec<RatatuiLine<'a>>,
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

    fn buffer_content<'a>(config: &Config, buffer_raw_ratatui_lines: Vec<RatatuiLine<'a>>) -> Paragraph<'a> {
        Paragraph::new(buffer_raw_ratatui_lines).fg(config.color_scheme.main.fg)
    }

    fn view_state<'a>(
        view_set: &'a mut ViewSet,
        buffer_map: &'a BufferMap,
        frame: &mut Frame,
        area_set: &AreaSet,
    ) -> Result<ViewState<'a>, AnyhowError> {
        let Some((active_view, buffer)) = view_set.active_pair_mut(buffer_map)? else {
            return ViewState::default().ok();
        };

        // NOTE-633c9d
        active_view
            .scroll_view_state_mut()
            .set_latest_content_render_size(area_set.buffer_content.as_size().into());

        let raw_ratatui_lines = buffer.raw_ratatui_lines(
            active_view.scroll_view_state().scroll_offset(),
            frame.area().as_size().into(),
        );
        let view_state = ViewState::new(active_view.name(), raw_ratatui_lines);

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

    #[tracing::instrument(skip_all)]
    fn draw_impl<'a>(
        &self,
        config: &Config,
        buffer_map: &'a BufferMap,
        view_set: &'a mut ViewSet,
        title: &mut Cow<'a, str>,
        frame: &mut Frame,
    ) -> Result<(), AnyhowError> {
        let area_set = self.area_set(frame.area());
        let mut timer = Timer::now();

        let (buffer_title, buffer_raw_ratatui_lines) = Self::view_state(view_set, buffer_map, frame, &area_set)?.into();

        timer.log("draw_impl(): Self::view_state(..)");

        Self::background(config).render_to(frame, area_set.background);
        Self::buffer_block(config, &buffer_title).render_to(frame, area_set.buffer_block);
        Self::buffer_content(config, buffer_raw_ratatui_lines).render_to(frame, area_set.buffer_content);

        timer.log("draw_impl(): *.render_to(..)");

        *title = buffer_title;

        ().ok()
    }

    #[tracing::instrument(skip_all)]
    pub fn draw(
        &self,
        config: &Config,
        buffer_map: &BufferMap,
        view_set: &mut ViewSet,
        terminal: &mut Terminal,
    ) -> Result<Vec<u8>, AnyhowError> {
        let mut title = Cow::default();
        let mut timer = Timer::now();

        terminal.draw(|frame| self.draw_impl(config, buffer_map, view_set, &mut title, frame))?;

        timer.log("draw(): terminal.draw(..)");

        terminal.set_title(title)?.take_bytes().ok()
    }
}
