use crate::{buffer_map::BufferMap, config::Config, view_set::ViewSet};
use anyhow::Error as AnyhowError;
use derive_more::Constructor;
use mkutils::{Default as MkutilsDefault, Terminal, Utils};
use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
};
use std::borrow::Cow;

#[derive(Constructor, MkutilsDefault)]
struct ViewState<'a> {
    #[default(Self::DEFAULT_TITLE.borrowed())]
    title: Cow<'a, str>,
    lines: Vec<Line<'a>>,
}

impl<'a> ViewState<'a> {
    const DEFAULT_TITLE: &'a str = "undefined";

    fn into_parts(self) -> (Cow<'a, str>, Paragraph<'a>) {
        let paragraph = self.lines.pipe_into(Paragraph::new);

        (self.title, paragraph)
    }
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
        Paragraph::default().fg(config.theme.main.fg).bg(config.theme.main.bg)
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

    fn area_set(&self, frame_area: Rect) -> AreaSet {
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

        // NOTE: need this mutable call on [view_set] to occur before i assign [buffer_title] to [title]
        view_set.set_latest_buffer_draw_size(area_set.buffer_content.as_size().into());

        let (buffer_title, buffer_content) = Self::view_state(view_set, buffer_map, frame)?.into_parts();

        Self::background(config).render_to(frame, area_set.background);
        buffer_title
            .as_borrowed::<str>()
            .bordered_block()
            .render_to(frame, area_set.buffer_block);
        buffer_content.render_to(frame, area_set.buffer_content);

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
