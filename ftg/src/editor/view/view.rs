use crate::{
    editor::{
        buffer::buffer::Buffer,
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
    const DOTS_WIDTH: u16 = 5;
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

    fn name(&self) -> &str {
        self.header.name()
    }

    fn render_title(&mut self, color_scheme: &ColorScheme) {
        let title_paragrah = self
            .header
            .title()
            .paragraph()
            .centered()
            .color(&color_scheme.title)
            .bold();

        self.terminal
            .render_widget(title_paragrah, self.terminal.area().width.row_at(0, 0));
    }

    fn dots(near_edge: bool, color_scheme: &ColorScheme) -> Paragraph<'static> {
        let dots = if near_edge { "" } else { "..." };

        dots.paragraph().centered().color(&color_scheme.tabs.dots)
    }

    fn render_tabs(&mut self, view_context: &ViewContext, color_scheme: &ColorScheme) {
        // NOTE-d7ec81
        let (num_possible_tabs, remainder_width) =
            (self.terminal.area().width - 2 * Self::DOTS_WIDTH).divmod(Self::TAB_WIDTH);
        let num_possible_tabs = num_possible_tabs as usize;
        let near_left_edge = (0..num_possible_tabs).contains(&view_context.index);
        let begin_idx_of_last_n = view_context.num_views.saturating_sub(num_possible_tabs);
        let near_right_edge = (begin_idx_of_last_n..view_context.num_views).contains(&view_context.index);
        let first_view_index_to_render = match (near_left_edge, near_right_edge) {
            (true, _) => 0,
            (_, true) => begin_idx_of_last_n,
            (false, false) => std::todo!(),
        };
        let enumerated_view_names = view_context
            .names(self.header.name())
            .enumerate()
            .skip(first_view_index_to_render)
            .take(num_possible_tabs);
        let left_dots = Self::dots(near_left_edge, color_scheme);
        let right_dots = Self::dots(near_right_edge, color_scheme);

        self.terminal.render_widget(left_dots, Self::DOTS_WIDTH.row_at(0, 1));

        // TODO: if num_possible tabs is 0, then terminal screen is too small to render, and i should
        // do dots
        let mut render_area_x = Self::DOTS_WIDTH;

        for (view_idx, view_name) in enumerated_view_names {
            let render_area = Self::TAB_WIDTH.row_at(render_area_x, 1);

            render_area_x = render_area_x.saturating_add(Self::TAB_WIDTH);

            let spec = if view_idx == view_context.index {
                &color_scheme.tabs.active
            } else if view_idx.is_even() {
                &color_scheme.tabs.primary
            } else {
                &color_scheme.tabs.secondary
            };
            let tab = view_name.paragraph().centered().color(spec);

            self.terminal.render_widget(tab, render_area);
        }

        for _empty_tab_idx in 0..num_possible_tabs.saturating_sub(view_context.num_views) {
            let render_area = Self::TAB_WIDTH.row_at(render_area_x, 1);

            render_area_x = render_area_x.saturating_add(Self::TAB_WIDTH);

            let empty_tab = Paragraph::default().color(&color_scheme.title);

            self.terminal.render_widget(empty_tab, render_area);
        }

        let remainder = Paragraph::default().color(&color_scheme.title);

        self.terminal
            .render_widget(remainder, remainder_width.row_at(render_area_x, 1));

        render_area_x = render_area_x.saturating_add(remainder_width);

        self.terminal
            .render_widget(right_dots, Self::DOTS_WIDTH.row_at(render_area_x, 1));
    }

    fn render_buffer(&mut self, buffer: &Buffer, color_scheme: &ColorScheme) {
        let buffer_area = self.terminal.area().saturating_sub_from_top(2);
        let sub_lines = buffer.sub_lines(&self.position, buffer_area);
        let mut selection_regions = self.selection_set.primary().iter();
        let mut selection_region_opt = selection_regions.next();
        let background = Paragraph::default().color(&color_scheme.buffer.unselected);

        self.terminal.render_widget(background, buffer_area);

        for (render_y, sub_line) in (2..).zip(sub_lines) {
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

            let spec = if selection_region_on_this_line {
                &color_scheme.buffer.selected
            } else {
                &color_scheme.buffer.unselected
            };

            let buffer_row = sub_line_spans.convert::<Line>().paragraph().color(spec);

            self.terminal
                .render_widget(buffer_row, buffer_area.width.row_at(0, render_y));
        }
    }

    pub fn render(
        &mut self,
        view_buffer_context: &ViewBufferContext,
        color_scheme: &ColorScheme,
    ) -> Result<Vec<u8>, Error> {
        self.render_title(color_scheme);
        self.render_tabs(&view_buffer_context.context, color_scheme);
        self.render_buffer(view_buffer_context.buffer, color_scheme);

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
