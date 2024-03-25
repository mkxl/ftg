use crate::{
    editor::{buffer::Buffer, client_state::ClientState, terminal::Terminal},
    error::Error,
    utils::any::Any,
};
use ratatui::{layout::Rect, widgets::Paragraph};
use ulid::Ulid;

#[derive(Default)]
struct Position {
    x: usize,
    y: usize,
}

pub struct View {
    buffer_id: Ulid,
    terminal: Terminal,
    position: Position,
}

impl View {
    pub fn new(buffer_id: Ulid, area: Rect) -> Result<Self, Error> {
        let terminal = Terminal::new(area);
        let position = Position::default();
        let view = Self {
            buffer_id,
            terminal,
            position,
        };

        view.ok()
    }

    pub fn buffer_id(&self) -> Ulid {
        self.buffer_id
    }

    pub fn move_down(&mut self) {
        self.position.y = self.position.y.saturating_add(1);
    }

    pub fn move_up(&mut self) {
        self.position.y = self.position.y.saturating_sub(1);
    }

    pub fn render(&mut self, client_state: &ClientState, buffer: &Buffer) -> Result<Vec<u8>, Error> {
        let paragraph = buffer.lines(self.position.y, client_state.config().size.1 as usize);
        let paragraph = Paragraph::new(paragraph);

        self.terminal.render_widget(paragraph, self.terminal.area());

        self.terminal.finish()
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.terminal.resize((width, height).rect());
    }
}
