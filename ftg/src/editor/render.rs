use crate::{
    editor::{buffer::buffer::Buffer, color_scheme::ColorScheme, terminal::Terminal, view::view::View},
    error::Error,
    utils::{any::Any, container::Container},
};
use ratatui::{style::Stylize, text::Line, widgets::Paragraph};

pub struct Render<'a> {
    terminal: &'a mut Terminal,
    views: &'a [View],
    view_index: usize,
    view: &'a View,
    buffer: &'a Buffer,
    color_scheme: &'a ColorScheme,
}

impl<'a> Render<'a> {
    const DOTS_WIDTH: u16 = 5;
    const TAB_WIDTH: u16 = 15;

    pub fn new(
        terminal: &'a mut Terminal,
        views: &'a [View],
        view_index: usize,
        buffers: &'a Container<Buffer>,
        color_scheme: &'a ColorScheme,
    ) -> Result<Self, Error> {
        let view = &views[view_index];
        let buffer = buffers.get(&view.buffer_id())?;
        let render = Self {
            terminal,
            views,
            view_index,
            view,
            buffer,
            color_scheme,
        };

        render.ok()
    }

    fn render_title(&mut self) {
        let title = self
            .view
            .header()
            .title()
            .paragraph()
            .centered()
            .color(&self.color_scheme.title)
            .bold();

        self.terminal
            .render_widget(title, self.terminal.area().width.row_at(0, 0));
    }

    fn dots(&self, near_edge: bool) -> Paragraph<'static> {
        let dots = if near_edge { "" } else { "..." };

        dots.paragraph().centered().color(&self.color_scheme.tabs.dots)
    }

    fn render_tabs(&mut self) {
        // NOTE-d7ec81
        let (num_possible_tabs, remainder_width) =
            (self.terminal.area().width - 2 * Self::DOTS_WIDTH).divmod(Self::TAB_WIDTH);
        let num_possible_tabs = num_possible_tabs as usize;
        let near_left_edge = (0..num_possible_tabs).contains(&self.view_index);
        let begin_idx_of_last_n = self.views.len().saturating_sub(num_possible_tabs);
        let near_right_edge = (begin_idx_of_last_n..self.views.len()).contains(&self.view_index);
        let first_view_index_to_render = match (near_left_edge, near_right_edge) {
            (true, _) => 0,
            (_, true) => begin_idx_of_last_n,
            (false, false) => std::todo!(),
        };
        let enumerated_view_names = self
            .views
            .iter()
            .map(|view| view.header().name())
            .enumerate()
            .skip(first_view_index_to_render)
            .take(num_possible_tabs);
        let left_dots = self.dots(near_left_edge);
        let right_dots = self.dots(near_right_edge);

        self.terminal.render_widget(left_dots, Self::DOTS_WIDTH.row_at(0, 1));

        // TODO: if num_possible tabs is 0, then terminal screen is too small to render, and i should just do dots
        let mut render_area_x = Self::DOTS_WIDTH;

        for (view_idx, view_name) in enumerated_view_names {
            let render_area = Self::TAB_WIDTH.row_at(render_area_x, 1);

            render_area_x = render_area_x.saturating_add(Self::TAB_WIDTH);

            let spec = if view_idx == self.view_index {
                &self.color_scheme.tabs.active
            } else if view_idx.is_even() {
                &self.color_scheme.tabs.primary
            } else {
                &self.color_scheme.tabs.secondary
            };
            let tab = view_name.paragraph().centered().color(spec);

            self.terminal.render_widget(tab, render_area);
        }

        for _empty_tab_idx in 0..num_possible_tabs.saturating_sub(self.views.len()) {
            let render_area = Self::TAB_WIDTH.row_at(render_area_x, 1);

            render_area_x = render_area_x.saturating_add(Self::TAB_WIDTH);

            let empty_tab = Paragraph::default().color(&self.color_scheme.title);

            self.terminal.render_widget(empty_tab, render_area);
        }

        let remainder = Paragraph::default().color(&self.color_scheme.title);

        self.terminal
            .render_widget(remainder, remainder_width.row_at(render_area_x, 1));

        render_area_x = render_area_x.saturating_add(remainder_width);

        self.terminal
            .render_widget(right_dots, Self::DOTS_WIDTH.row_at(render_area_x, 1));
    }

    fn render_buffer(&mut self) {
        let buffer_area = self.terminal.area().saturating_sub_from_top(2);
        let sub_lines = self.buffer.sub_lines(self.view.position(), buffer_area);
        let mut selection_regions = self.view.selection_set().primary().iter();
        let mut selection_region_opt = selection_regions.next();
        let background = Paragraph::default().color(&self.color_scheme.buffer.unselected);

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
                &self.color_scheme.buffer.selected
            } else {
                &self.color_scheme.buffer.unselected
            };

            let buffer_row = sub_line_spans.convert::<Line>().paragraph().color(spec);

            self.terminal
                .render_widget(buffer_row, buffer_area.width.row_at(0, render_y));
        }
    }

    pub fn render(&mut self) -> Result<Vec<u8>, Error> {
        self.render_title();
        self.render_tabs();
        self.render_buffer();

        self.terminal.finish()
    }
}
