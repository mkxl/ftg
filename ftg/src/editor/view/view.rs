use crate::{
    editor::{
        buffer::buffer::{Buffer, SubLine},
        color_scheme::ColorScheme,
        keymap::Context,
        selection::{region::Region, selection::Selection, set::SelectionSet},
        terminal::Terminal,
        view::{header::Header, search::Search},
    },
    error::Error,
    utils::{any::Any, container::Identifiable, position::Position},
};
use ratatui::{layout::Rect, style::Stylize, text::Line, widgets::Paragraph};
use std::{io::Error as IoError, path::PathBuf};
use ulid::Ulid;

pub struct ViewContext<'a> {
    pub before: &'a [View],
    pub after: &'a [View],
    pub index: usize,
    pub num_views: usize,
}

impl<'a> ViewContext<'a> {
    pub fn names(&'a self, name: &'a str) -> impl Iterator<Item = &'a str> {
        let left = self.before.iter().map(View::name);
        let right = self.after.iter().map(View::name);

        left.chain_once(name).chain(right)
    }
}

pub struct ViewBufferContext<'a> {
    pub buffer: &'a mut Buffer,
    pub context: ViewContext<'a>,
}

pub struct View {
    id: Ulid,
    buffer_id: Ulid,
    terminal: Terminal,
    position: Position,
    header: Header,
    selection_set: SelectionSet,
    context: Context,
    search: Search,
}

impl View {
    const WIDTH_DOTS: u16 = 5;
    const TAB_WIDTH: u16 = 15;

    pub fn new(buffer_id: Ulid, rect: Rect, filepath: Option<PathBuf>) -> Result<Self, Error> {
        let id = Ulid::new();
        let terminal = Terminal::new(rect);
        let header = Header::new(filepath);
        let position = Position::zero();
        let selection_set = Region::unit(0).into();
        let context = Context::Buffer;
        let search = Search::default();
        let view = Self {
            id,
            buffer_id,
            terminal,
            position,
            header,
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
        color_scheme: &'a ColorScheme,
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
            let mut selection_region_on_this_line = false;

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

                selection_region_on_this_line = true;

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
                    .bold()
                    .push_to(&mut sub_line_spans);

                // NOTE: update the current sub_line remainder so that it begins after the end of the intersection
                sub_line_sub_region_opt = sub_line_sub_region.with_start(intersection.end_exclusive());
            }

            let line = sub_line_spans
                .convert::<Line<'a>>()
                .fg(color_scheme.buffer.fg)
                .bg(color_scheme.buffer.bg);

            if selection_region_on_this_line {
                line.reversed()
            } else {
                line
            }
        })
    }

    fn name(&self) -> &str {
        self.header.name()
    }

    fn tab_line<'a>(
        terminal_area: Rect,
        view_context: &'a ViewContext<'a>,
        active_view_name: &'a str,
        color_scheme: &ColorScheme,
    ) -> Result<Line<'static>, Error> {
        // NOTE-d7ec81
        let (num_possible_tabs, _width_rem) = (terminal_area.width - 2 * Self::WIDTH_DOTS).divmod(Self::TAB_WIDTH);
        let num_possible_tabs = num_possible_tabs as usize;
        let begin_idx_of_last_n = view_context.num_views.saturating_sub(num_possible_tabs);
        let first_view_index_to_render = if (0..num_possible_tabs).contains(&view_context.index) {
            0
        } else if (begin_idx_of_last_n..view_context.num_views).contains(&view_context.index) {
            begin_idx_of_last_n
        } else {
            std::todo!()
        };
        let line = view_context
            .names(active_view_name)
            .enumerate()
            .skip(first_view_index_to_render)
            .take(num_possible_tabs)
            .map(|(idx, view_name)| {
                let spec = if idx == view_context.index {
                    &color_scheme.tabs.active
                } else if idx.is_even() {
                    &color_scheme.tabs.primary
                } else {
                    &color_scheme.tabs.secondary
                };

                // TODO: don't allocate a string, but yield an itereator, flatten the iterator of iterators, and
                // collect into a line
                view_name
                    .center(" ", Self::TAB_WIDTH.into())
                    .collect::<String>()
                    .fg(spec.fg)
                    .fg(spec.bg)
            })
            .collect::<Line>();

        line.ok()
    }

    pub fn render(
        &mut self,
        view_buffer_context: &ViewBufferContext,
        color_scheme: &ColorScheme,
    ) -> Result<Vec<u8>, Error> {
        // TODO: can i make all these Self::*() methods self.*() methods?
        let terminal_area = self.terminal.area();
        let title_line = self
            .header
            .title()
            .convert::<Line>()
            .centered()
            .fg(color_scheme.title.fg)
            .bg(color_scheme.title.bg)
            .bold();
        let tab_line = Self::tab_line(
            terminal_area,
            &view_buffer_context.context,
            self.header.name(),
            color_scheme,
        )?;
        let buffer_area = terminal_area.saturating_sub(0, 2);
        let sub_lines = view_buffer_context.buffer.sub_lines(&self.position, buffer_area);
        let lines = Self::lines(&self.selection_set, sub_lines, color_scheme);
        let lines = title_line.once().chain_once(tab_line).chain(lines).collect::<Vec<_>>();
        let paragraph = Paragraph::new(lines);

        self.terminal.render_widget(paragraph, terminal_area);

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
        let Some(filepath) = &self.header.path() else {
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
