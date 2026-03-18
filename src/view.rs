use crate::{buffer::Buffer, view_state::ViewState};
use derive_more::Constructor;
use mkutils::{ScrollView, ScrollViewState, Utils};
use ratatui::{style::Modifier, text::Line as RatatuiLine};
use std::ops::Range;

#[derive(Constructor)]
pub struct View<'a> {
    view_state: &'a mut ViewState,
    buffer: &'a mut Buffer,
}

impl View<'_> {
    pub const fn view_state(&self) -> &ViewState {
        self.view_state
    }

    pub fn move_forward(&mut self) {
        self.view_state
            .selection_set_mut()
            .active_vec_mut()
            .active_mut()
            .translate(1);
    }

    pub fn move_backward(&mut self) {
        self.view_state
            .selection_set_mut()
            .active_vec_mut()
            .active_mut()
            .translate(-1);
    }

    pub fn replace(&mut self, text: &str) {
        let rope = self.buffer.rope_mut();
        let selection = self.view_state.selection_set_mut().active_vec_mut().active_mut();

        for region in selection.regions().rev() {
            rope.replace(region.range(), text);
        }

        selection.translate(text.extended_graphemes().count().cast_or_max());
    }
}

impl ScrollView for View<'_> {
    fn ratatui_lines(&self, rows: Range<usize>, cols: Range<usize>) -> Vec<RatatuiLine<'_>> {
        let mut rope_lines = self.buffer.rope().lines(rows, cols);
        let mut raw_ratatui_lines = Vec::new();

        while let Some(rope_line) = rope_lines.next_line() {
            rope_line
                .map(|atom| {
                    let extended_grapheme = atom.extended_grapheme();
                    let extended_grapheme = extended_grapheme.is_newline().if_else(" ", extended_grapheme);

                    self.view_state
                        .selection_set()
                        .active_vec()
                        .active()
                        .contains_point(atom.offset().extended_graphemes().get())
                        .if_else(Modifier::REVERSED, Modifier::empty())
                        .stylize(extended_grapheme)
                })
                .collect::<RatatuiLine>()
                .push_to(&mut raw_ratatui_lines);
        }

        raw_ratatui_lines
    }

    fn scroll_view_state_mut(&mut self) -> &mut ScrollViewState {
        self.view_state.scroll_view_state_mut()
    }
}
