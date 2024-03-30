use crate::{
    editor::{buffer::Buffer, terminal::Terminal, window::Window},
    error::Error,
    utils::{any::Any, container::Identifiable},
};
use ratatui::{layout::Rect, widgets::Paragraph};
use ulid::Ulid;

#[derive(Debug, Default)]
struct Position {
    x: usize,
    y: usize,
}

pub struct View {
    id: Ulid,
    buffer_id: Ulid,
    terminal: Terminal,
    position: Position,
}

impl View {
    pub fn new(buffer_id: Ulid, area: Rect) -> Result<Self, Error> {
        let id = Ulid::new();
        let terminal = Terminal::new(area);
        let position = Position::default();
        let view = Self {
            id,
            buffer_id,
            terminal,
            position,
        };

        view.ok()
    }

    pub fn buffer_id(&self) -> Ulid {
        self.buffer_id
    }

    pub fn move_down(&mut self, buffer: &Buffer) {
        let max_y = buffer.len_lines().saturating_sub(2);

        self.position.y = self.position.y.saturating_add(1).min(max_y);
    }

    pub fn move_up(&mut self) {
        self.position.y = self.position.y.saturating_sub(1);
    }

    pub fn render(&mut self, _window: &Window, buffer: &Buffer) -> Result<Vec<u8>, Error> {
        let paragraph = buffer.lines(self.position.y, self.terminal.area().height as usize);
        let paragraph = Paragraph::new(paragraph);

        self.terminal.render_widget(paragraph, self.terminal.area());

        self.terminal.finish()
    }

    pub fn resize(&mut self, width: u16, height: u16) -> Result<(), Error> {
        self.terminal.resize((width, height).rect())
    }
}

impl Identifiable for View {
    fn id(&self) -> Ulid {
        self.id
    }
}
