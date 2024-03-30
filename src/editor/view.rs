use crate::{
    editor::{
        buffer::Buffer,
        terminal::Terminal,
        window::{Args as WindowArgs, Window},
    },
    error::Error,
    utils::{any::Any, container::Identifiable},
};
use ratatui::{text::Line, widgets::Paragraph};
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
    args: WindowArgs,
}

impl View {
    const DEFAULT_TITLE: &'static str = "Untitled";

    pub fn new(buffer_id: Ulid, args: WindowArgs) -> Result<Self, Error> {
        let id = Ulid::new();
        let terminal = Terminal::new(args.size.rect());
        let position = Position::default();
        let view = Self {
            id,
            buffer_id,
            terminal,
            position,
            args,
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
        let title = if let Some(filepath) = &self.args.filepath {
            Line::raw(filepath.display().to_string())
        } else {
            Line::raw(Self::DEFAULT_TITLE)
        };
        let count = self.terminal.area().height.saturating_sub(1).convert::<usize>();
        // TODO: replace `.map(|rope_slice| rope_slice.to_string())` with non closure function
        let lines = buffer
            .lines(self.position.y, count)
            .map(|rope_slice| rope_slice.to_string())
            .map(Line::raw);
        let lines = Some(title).into_iter().chain(lines).collect::<Vec<_>>();
        let paragraph = Paragraph::new(lines);

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
