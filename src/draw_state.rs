use crate::{buffer_map::BufferMap, config::Config, views::ViewStateSet};
use anyhow::Error as AnyhowError;
use mkutils::{ScrollView, Terminal, Utils};
use ratatui::{Frame, layout::Rect, style::Stylize, widgets::Paragraph};
use std::borrow::Cow;

struct AreaSet {
    background: Rect,
    buffer_content: Rect,
}

#[derive(Default)]
pub struct DrawState;

impl DrawState {
    const EMPTY_VIEWS_TITLE: &str = "undefined";

    const fn area_set(&self, frame_area: Rect) -> AreaSet {
        let background = frame_area;
        let buffer_content = frame_area;

        AreaSet {
            background,
            buffer_content,
        }
    }

    fn background(config: &Config) -> Paragraph<'static> {
        Paragraph::default()
            .fg(config.color_scheme.main.fg)
            .bg(config.color_scheme.main.bg)
    }

    fn empty_views() -> Paragraph<'static> {
        Paragraph::default()
    }

    pub fn draw_impl<'a>(
        &self,
        config: &Config,
        buffer_map: &'a BufferMap,
        view_state_set: &'a mut ViewStateSet,
        title: &mut Cow<'a, str>,
        frame: &mut Frame,
    ) -> Result<(), AnyhowError> {
        let area_set = self.area_set(frame.area());

        Self::background(config).render_to(frame, area_set.background);

        if let Some(mut view) = view_state_set.active_view_mut(buffer_map)? {
            view.render(frame, area_set.buffer_content);
            title.assign(view.title().to_owned().into_cow_owned())
        } else {
            Self::empty_views().render_to(frame, area_set.buffer_content);
            title.assign(Self::EMPTY_VIEWS_TITLE.to_cow_borrowed());
        }

        ().ok()
    }

    pub fn draw(
        &self,
        config: &Config,
        buffer_map: &BufferMap,
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
