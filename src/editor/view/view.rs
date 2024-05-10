use crate::{
    editor::{
        buffer::buffer::{Buffer, Chunk},
        keymap::Context,
        selection::{region::Region, set::SelectionSet},
        terminal::Terminal,
        view::search::Search,
        window::{Window, WindowArgs},
    },
    error::Error,
    utils::{any::Any, container::Identifiable, position::Position},
};
use ratatui::{style::Stylize, text::Line, widgets::Paragraph};
use std::{ffi::OsStr, path::Path};
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
        let selection_set = Region::ii(0, 0).unwrap().into();
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

    fn lines<'a, C: 'a + Iterator<Item = Chunk<'a>>>(
        selection_set: &'a SelectionSet,
        chunks: C,
    ) -> impl 'a + Iterator<Item = Line<'a>> {
        let mut selection_regions = selection_set.primary().iter();
        let mut selection_region_opt = selection_regions.next();

        chunks.map(move |chunk| {
            // NOTE:
            // - we call chunk.chars() and process the Chars iterator to avoid the O(log N) [1] cost of having to index
            //   into the chunk rope slice multiple times
            // - see [2] for the source of the implementation
            // - [1]: [https://docs.rs/ropey/latest/ropey/struct.Rope.html#method.slice]
            // - [2]: [~/tree/projects/.scratch/python/notebooks/2024-05-07-chunks.ipynb]
            let mut chunk_subregion_opt = chunk.region();
            let mut chunk_chars = chunk.chars();
            let mut chunk_spans = std::vec![];

            loop {
                // NOTE: if the current chunk remainder is empty, then i'm done processing the chunk, and i can
                // continue onto the next chunk
                let Some(chunk_subregion) = chunk_subregion_opt else {
                    break;
                };

                // NOTE: if there are no more selection regions, yield the current chunk remainder and continue
                // onto the next chunk
                let Some(selection_region) = &selection_region_opt else {
                    chunk_chars.span(chunk_subregion.len()).push_to(&mut chunk_spans);

                    break;
                };

                let Some(intersection) = selection_region.intersect(&chunk_subregion) else {
                    // NOTE: chunk_subregion is nonempty, so if the intersection is empty and selection_region
                    // is to the left of chunk_subregion, then selection_region must end before chunk_subregion
                    // begins, and i can skip to the next selection_region
                    if selection_region.start() < chunk_subregion.start() {
                        selection_region_opt = selection_regions.next();

                        continue;
                    }

                    // NOTE: otherwise, selection_region is to the right of chunk_subregion, and i can skip to the next
                    // chunk after first yielding the current chunk remainder
                    chunk_chars.span(chunk_subregion.len()).push_to(&mut chunk_spans);

                    break;
                };

                // NOTE: if the beginning of the current chunk remainder is not included in the intersection, yield it
                if chunk_subregion.start() < intersection.start() {
                    chunk_chars
                        .span(intersection.start() - chunk_subregion.start())
                        .push_to(&mut chunk_spans);
                }

                // NOTE: yield the intersection
                chunk_chars
                    .span(intersection.len())
                    .reversed()
                    .push_to(&mut chunk_spans);

                // NOTE: update the current chunk remainder so that it begins after the end of the intersection
                chunk_subregion_opt = chunk_subregion.with_start(intersection.end_exclusive());
            }

            chunk_spans.into()
        })
    }

    pub fn render(&mut self, _window: &Window, buffer: &Buffer) -> Result<Vec<u8>, Error> {
        let title_line = self
            .args
            .filepath
            .as_deref()
            .and_then(Path::file_name)
            .and_then(OsStr::to_str)
            .unwrap_or(Self::DEFAULT_TITLE)
            .reversed()
            .convert::<Line>();
        let area = self.terminal.area().saturating_sub(0, 1);
        let chunks = buffer.chunks(&self.position, area);
        let lines = Self::lines(&self.selection_set, chunks);
        let lines = title_line.once().chain(lines).collect::<Vec<_>>();
        let paragraph = Paragraph::new(lines);

        self.terminal.render_widget(paragraph, self.terminal.area());

        self.terminal.finish()
    }

    pub fn resize(&mut self, width: u16, height: u16) -> Result<(), Error> {
        self.terminal.resize((width, height).rect())
    }

    pub fn context(&self) -> Context {
        self.context
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
        self.selection_set = buffer.search(self.search.query()).collect();

        self.close_search();
        self.search.clear();
    }

    pub fn close_search(&mut self) {
        self.context = Context::Buffer;
    }

    pub fn insert_character(&mut self, buffer: &mut Buffer, chr: char) {
        tracing::info!(message = "insert_character", position = ?self.position, char = %chr);
    }
}

impl Identifiable for View {
    fn id(&self) -> Ulid {
        self.id()
    }
}
