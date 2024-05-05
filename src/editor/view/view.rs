use crate::{
    editor::{
        buffer::buffer::Buffer,
        keymap::Context,
        selection::set::SelectionSet,
        terminal::Terminal,
        view::search::Search,
        window::{Window, WindowArgs},
    },
    error::Error,
    utils::{any::Any, container::Identifiable, position::Position},
};
use ratatui::{style::Stylize, text::Line, widgets::Paragraph};
use ulid::Ulid;

pub struct View {
    id: Ulid,
    buffer_id: Ulid,
    terminal: Terminal,
    position: Position,
    args: WindowArgs,
    selection_set: SelectionSet,
    context: Context,
    search: Search,
}

impl View {
    const DEFAULT_TITLE: &'static str = "Untitled";

    pub fn new(buffer_id: Ulid, args: WindowArgs) -> Result<Self, Error> {
        let id = Ulid::new();
        let terminal = Terminal::new(args.size.rect());
        let position = Position::zero();
        let selection_set = SelectionSet::default();
        let context = Context::Buffer;
        let search = Search::default();
        let view = Self {
            id,
            buffer_id,
            terminal,
            position,
            args,
            selection_set,
            context,
            search,
        };

        view.ok()
    }

    pub fn id(&self) -> Ulid {
        self.id
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

    pub fn move_left(&mut self) {
        self.position.x = self.position.x.saturating_sub(1);
    }

    // TODO: need to find max value (requires getting length of each line)
    pub fn move_right(&mut self) {
        self.position.x = self.position.x.saturating_add(1);
    }

    pub fn render(&mut self, _window: &Window, buffer: &Buffer) -> Result<Vec<u8>, Error> {
        let title = if let Some(filepath) = &self.args.filepath {
            filepath.display().to_string().reversed()
        } else {
            Self::DEFAULT_TITLE.reversed()
        }
        .convert::<Line<'_>>();
        let area = self.terminal.area().saturating_sub(0, 1);
        let lines = buffer.lines(&self.position, area).map(Line::raw);
        let lines = title.once().chain(lines).collect::<Vec<_>>();
        let paragraph = Paragraph::new(lines);

        self.terminal.render_widget(paragraph, self.terminal.area());

        self.terminal.finish()
    }

    pub fn resize(&mut self, width: u16, height: u16) -> Result<(), Error> {
        self.terminal.resize((width, height).rect())
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn begin_search(&mut self) {
        self.context = Context::Search;
    }

    pub fn push_search(&mut self, chr: char) {
        self.search.push(chr);

        // TODO: remove
        tracing::info!(view.search.query = ?self.search.query());
    }

    pub fn submit_search(&mut self, buffer: &Buffer) {
        for region in buffer.search(self.search.query()) {
            // TODO: remove
            tracing::info!(?region);
        }
    }

    pub fn close_search(&mut self) {
        self.context = Context::Buffer;
    }
}

impl Identifiable for View {
    fn id(&self) -> Ulid {
        self.id()
    }
}
