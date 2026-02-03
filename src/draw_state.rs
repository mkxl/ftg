use crate::{buffer_map::BufferMap, config::Config, view::View};
use anyhow::Error as AnyhowError;
use derive_more::Constructor;
use mkutils::{ActiveVec, Default as MkutilsDefault, Terminal, Utils};
use ratatui::{Frame, layout::Rect, style::Stylize, text::Line, widgets::Paragraph};
use std::borrow::{Borrow, Cow};

#[derive(Constructor, MkutilsDefault)]
struct ViewState<'a> {
    #[default(Self::DEFAULT_TITLE.borrowed())]
    title: Cow<'a, str>,
    lines: Vec<Line<'a>>,
}

impl ViewState<'_> {
    fn take_widget(&mut self) -> Paragraph<'_> {
        Paragraph::new(self.lines.mem_take()).block(self.title.as_borrowed::<str>().bordered_block())
    }
}

impl<'a> ViewState<'a> {
    const DEFAULT_TITLE: &'a str = "undefined";
}

struct AreaSet {
    background: Rect,
}

#[derive(Default)]
pub struct DrawState;

impl DrawState {
    fn background(config: &Config) -> Paragraph<'static> {
        Paragraph::default().fg(config.theme.main.fg).bg(config.theme.main.bg)
    }

    fn view_state<'a>(
        views: Option<&'a ActiveVec<View>>,
        buffer_map: &'a BufferMap,
        frame: &mut Frame,
    ) -> Result<ViewState<'a>, AnyhowError> {
        let Some(views) = views else { return ViewState::default().ok() };
        let active_view = views.active();
        let lines = buffer_map
            .get(&active_view.buffer_id())
            .check_present()?
            .lines(active_view.scroll_offset(), frame.area().as_size().into());
        let view_state = ViewState::new(active_view.name(), lines);

        view_state.ok()
    }

    fn area_set(&self, frame_area: Rect) -> AreaSet {
        let background = frame_area;

        AreaSet { background }
    }

    fn draw_impl<'a>(
        &self,
        config: &Config,
        buffer_map: &'a BufferMap,
        views: Option<&'a ActiveVec<View>>,
        title: &mut Cow<'a, str>,
        frame: &mut Frame,
    ) -> Result<(), AnyhowError> {
        let mut view_state = Self::view_state(views, buffer_map, frame)?;
        let area_set = self.area_set(frame.area());

        Self::background(config).render_to(frame, area_set.background);
        view_state.take_widget().render_to(frame, area_set.background);

        *title = view_state.title;

        ().ok()
    }

    pub fn draw(
        &self,
        config: &Config,
        buffer_map: &BufferMap,
        views: Option<&ActiveVec<View>>,
        terminal: &mut Terminal,
    ) -> Result<Vec<u8>, AnyhowError> {
        let mut title = Cow::default();

        terminal
            .draw(|frame| self.draw_impl(config, buffer_map, views, &mut title, frame).io_result())?
            .set_title(title)?
            .take_byte_str()
            .ok()
    }
}
