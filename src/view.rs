use crate::{buffer::Buffer, selection::Selection, view_state::ViewState};
use derive_more::Constructor;
use mkutils::{Interval, Rope, ScrollView, ScrollViewState, Utils};
use ratatui::{style::Modifier, text::Line as RatatuiLine};
use std::ops::Range;

#[derive(Constructor)]
pub struct View<'a> {
    view_state: &'a mut ViewState,
    buffer: &'a mut Buffer,
}

impl View<'_> {
    pub const fn buffer(&self) -> &Buffer {
        self.buffer
    }

    pub const fn view_state(&self) -> &ViewState {
        self.view_state
    }

    pub fn move_forward(&mut self) {
        self.view_state.selections_mut().active_mut().translate(1);
    }

    pub fn move_backward(&mut self) {
        self.view_state.selections_mut().active_mut().translate(-1);
    }

    pub fn move_up(&mut self) {
        std::todo!();
    }

    pub fn move_down(&mut self) {
        std::todo!();
    }

    pub fn buffer_rope_and_active_selection(&mut self) -> (&mut Rope, &mut Selection) {
        (self.buffer.rope_mut(), self.view_state.selections_mut().active_mut())
    }

    pub fn replace(&mut self, text: &str) {
        let (buffer_rope, active_selection) = self.buffer_rope_and_active_selection();

        for region in active_selection.regions().rev() {
            buffer_rope.replace(region.range(), text);
        }

        active_selection.translate(text.extended_graphemes().count().cast_or_max());
    }

    pub fn delete(&mut self) {
        let (buffer_rope, active_selection) = self.buffer_rope_and_active_selection();
        let mut new_regions = Vec::new();

        for region in active_selection.regions().rev() {
            let delete_range = if region.is_empty() && !region.begins_at(&0) {
                region.translated(-1).expanded(1).range()
            } else {
                region.range()
            };

            buffer_rope.delete(delete_range);
            new_regions.push(region.begin().decremented().into());
        }

        if let Some(new_active_selection) = Selection::from_regions(new_regions) {
            self.view_state.set_active_selection(new_active_selection);
        }
    }

    pub fn line_num_width(&self) -> usize {
        self.buffer.rope().size().y.ilog10().incremented().cast_or_max()
    }
}

impl ScrollView for View<'_> {
    fn ratatui_lines(&self, rows: Range<usize>, cols: Range<usize>) -> Vec<RatatuiLine<'_>> {
        let mut rope_lines = self.buffer.rope().lines(rows, cols);
        let mut ratatui_lines = Vec::new();
        let active_selection = self.view_state.selections().active();

        while let Some(rope_line) = rope_lines.next_line() {
            rope_line
                .map(|atom| {
                    let extended_grapheme = atom.extended_grapheme();
                    let extended_grapheme = extended_grapheme.is_newline().if_else(" ", extended_grapheme);

                    active_selection
                        .contains_point(atom.offset().extended_graphemes().get())
                        .if_else(Modifier::REVERSED, Modifier::empty())
                        .stylize(extended_grapheme)
                })
                .collect::<RatatuiLine>()
                .push_to(&mut ratatui_lines);
        }

        ratatui_lines
    }

    fn scroll_view_state_mut(&mut self) -> &mut ScrollViewState {
        self.view_state.scroll_view_state_mut()
    }
}
