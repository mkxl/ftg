use crate::{buffer::Buffer, region::Region, selection::Selection, view_state::ViewState};
use derive_more::Constructor;
use mkutils::{Interval, PointIsize, Rope, ScrollView, ScrollViewState, Utils};
use num::traits::{ConstOne, ConstZero};
use ratatui::{Frame, layout::Rect, style::Modifier, text::Line as RatatuiLine};
use std::ops::Neg;

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
        self.view_state.selections_mut().active_mut().translate(isize::ONE);
    }

    pub fn move_backward(&mut self) {
        self.view_state.selections_mut().active_mut().translate(-isize::ONE);
    }

    fn move_with(&mut self, point_offset: PointIsize) {
        let translated = |region: &Region| {
            let offset = self.buffer.rope().translated(region.begin(), &point_offset);
            let count = offset.text_summary.length.extended_graphemes.cast_or_max::<isize>();
            let count = offset.is_forward.if_else(count, count.neg());

            region.translated(count)
        };
        let new_regions = self.view_state.selections().active().regions().map(translated);

        if let Some(new_active_selection) = Selection::from_regions(new_regions) {
            self.view_state.set_active_selection(new_active_selection);
        }
    }

    pub fn move_up(&mut self) {
        self.move_with(isize::ONE.neg().into_point_y());
    }

    pub fn move_down(&mut self) {
        self.move_with(isize::ONE.into_point_y());
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
            let delete_range = if region.is_empty() && !region.begins_at(&usize::ZERO) {
                region.translated(-isize::ONE).expanded(isize::ONE).range()
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
    fn render_content(&self, frame: &mut Frame, content_area: Rect) {
        let scroll_offset = self.scroll_view_state().scroll_offset();
        let rows = scroll_offset.y.range_from_len(content_area.height.into());
        let cols = scroll_offset.x.range_from_len(content_area.width.into());
        let mut rope_lines = self.buffer.rope().lines(rows, cols);
        let mut row_areas = content_area.rows();
        let active_selection = self.view_state.selections().active();

        while let Some(rope_line) = rope_lines.next_line() {
            let Some(row_area) = row_areas.next() else { break };

            rope_line
                .map(|atom| {
                    let extended_grapheme = atom.extended_grapheme.is_newline().if_else(" ", atom.extended_grapheme);

                    active_selection
                        .contains_point(atom.offset.length.extended_graphemes)
                        .if_else(Modifier::REVERSED, Modifier::empty())
                        .stylize(extended_grapheme)
                })
                .collect::<RatatuiLine>()
                .render_to(frame, row_area);
        }
    }

    fn scroll_view_state(&self) -> &ScrollViewState {
        self.view_state.scroll_view_state()
    }

    fn scroll_view_state_mut(&mut self) -> &mut ScrollViewState {
        self.view_state.scroll_view_state_mut()
    }
}
