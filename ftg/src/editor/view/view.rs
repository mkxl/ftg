use crate::{
    editor::{
        buffer::buffer::{Buffer, SubLine},
        keymap::Context,
        selection::{region::Region, selection::Selection, set::SelectionSet},
        terminal::Terminal,
        view::search::Search,
        window::{Window, WindowArgs},
    },
    error::Error,
    utils::{any::Any, container::Identifiable, position::Position},
};
use ratatui::{style::Stylize, text::Line, widgets::Paragraph};
use std::{ffi::OsStr, io::Error as IoError, path::Path};
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
        let selection_set = Region::unit(0).into();
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

    pub fn move_down(&mut self, buffer: &Buffer, count: usize) {
        let max_y = buffer.len_lines().saturating_sub(2);

        self.position.y = self.position.y.saturating_add(count).min(max_y);
    }

    pub fn move_up(&mut self, count: usize) {
        self.position.y = self.position.y.saturating_sub(count);
    }

    pub fn move_left(&mut self) {
        self.position.x = self.position.x.saturating_sub(1);
    }

    // TODO: need to find max value (requires getting length of each line that's being rendered)
    pub fn move_right(&mut self) {
        self.position.x = self.position.x.saturating_add(1);
    }

    fn lines<'a, I: 'a + Iterator<Item = SubLine<'a>>>(
        selection_set: &'a SelectionSet,
        sub_lines: I,
    ) -> impl 'a + Iterator<Item = Line<'a>> {
        let mut selection_regions = selection_set.primary().iter();
        let mut selection_region_opt = selection_regions.next();

        sub_lines.map(move |sub_line| {
            // NOTE-ad63f1:
            // - we call sub_line.chars() and process the Chars iterator to avoid the O(log N) [1] cost of having to
            //   index into the sub_line rope slice multiple times
            // - see [2] for the source of the implementation
            // - [1]: [https://docs.rs/ropey/latest/ropey/struct.Rope.html#method.slice]
            // - [2]: [~/tree/projects/.scratch/python/notebooks/2024-05-07-chunks.ipynb]
            let mut sub_line_sub_region_opt = sub_line.region();
            let mut sub_line_chars = sub_line.chars();
            let mut sub_line_spans = std::vec![];

            loop {
                // NOTE: if the current sub_line remainder is empty, then i'm done processing the sub_line, and i can
                // continue onto the next sub_line
                let Some(sub_line_sub_region) = sub_line_sub_region_opt else {
                    break;
                };

                // NOTE: if there are no more selection regions, yield the current sub_line remainder and continue
                // onto the next sub_line
                let Some(selection_region) = &selection_region_opt else {
                    sub_line_chars
                        .span(sub_line_sub_region.len())
                        .push_to(&mut sub_line_spans);

                    break;
                };

                let Some(intersection) = selection_region.intersect(&sub_line_sub_region) else {
                    // NOTE: sub_line_sub_region is nonempty, so if the intersection is empty and selection_region
                    // is to the left of sub_line_sub_region, then selection_region must end before sub_line_sub_region
                    // begins, and i can skip to the next selection_region
                    if selection_region.start() < sub_line_sub_region.start() {
                        selection_region_opt = selection_regions.next();

                        continue;
                    }

                    // NOTE: otherwise, selection_region is to the right of sub_line_sub_region, and i can skip to the
                    // next sub_line after first yielding the current sub_line remainder
                    sub_line_chars
                        .span(sub_line_sub_region.len())
                        .push_to(&mut sub_line_spans);

                    break;
                };

                // NOTE: if the beginning of the current sub_line remainder is not included in the intersection, yield
                // it
                if sub_line_sub_region.start() < intersection.start() {
                    sub_line_chars
                        .span(intersection.start() - sub_line_sub_region.start())
                        .push_to(&mut sub_line_spans);
                }

                // NOTE: yield the intersection
                sub_line_chars
                    .span(intersection.len())
                    .reversed()
                    .push_to(&mut sub_line_spans);

                // NOTE: update the current sub_line remainder so that it begins after the end of the intersection
                sub_line_sub_region_opt = sub_line_sub_region.with_start(intersection.end_exclusive());
            }

            sub_line_spans.into()
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
        let sub_lines = buffer.sub_lines(&self.position, area);
        let lines = Self::lines(&self.selection_set, sub_lines);
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

    pub fn insert_char(&mut self, buffer: &mut Buffer, chr: char) {
        let selection = self.selection_set.primary_mut();
        let mut new_selection = Selection::default();
        let len_chars = 1;

        for (region_idx, region) in selection.iter().enumerate() {
            let diff = region_idx.saturating_mul(len_chars);
            let insert_idx = region.start().saturating_add(diff);
            let new_region = Region::unit(insert_idx.saturating_add(1));

            buffer.insert_char(insert_idx, chr).warn();
            new_selection.insert(new_region);
        }

        selection.replace_with(new_selection);
    }

    pub fn save(&self, buffer: &Buffer) -> Result<(), IoError> {
        let Some(filepath) = self.args.filepath.as_ref() else {
            return ().ok();
        };

        filepath.create()?.buf_writer().write_iter(buffer.chunks())?.ok()
    }
}

impl Identifiable for View {
    fn id(&self) -> Ulid {
        self.id()
    }
}
