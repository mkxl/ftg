use crate::{
    editor::{
        buffer::buffer::Buffer,
        keymap::Context,
        selection::{region::Region, selection::Selection, set::SelectionSet},
        view::{header::Header, search::Search},
    },
    error::Error,
    utils::{any::Any, container::Identifiable, path::Path, position::Position},
};
use std::io::Error as IoError;
use ulid::Ulid;

pub struct View {
    id: Ulid,
    buffer_id: Ulid,
    position: Position,
    header: Header,
    selection_set: SelectionSet,
    context: Context,
    search: Search,
}

impl View {
    pub fn new(buffer_id: Ulid, filepath: Option<Path>) -> Result<Self, Error> {
        let id = Ulid::new();
        let header = Header::new(filepath);
        let position = Position::zero();
        let selection_set = Region::unit(0).into();
        let context = Context::Buffer;
        let search = Search::default();
        let view = Self {
            id,
            buffer_id,
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

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn selection_set(&self) -> &SelectionSet {
        &self.selection_set
    }

    pub fn context(&self) -> Context {
        self.context
    }

    fn translate_by(&mut self, count: isize) {
        let selection = self.selection_set.primary_mut();

        *selection = selection.iter().map(|region| region.translate_by(count)).collect();
    }

    pub fn translate_by_line(&mut self, buffer: &Buffer, count: isize) {
        let selection = self.selection_set.primary_mut();

        *selection = selection
            .iter()
            .map(|region| {
                let (row, col) = buffer.row_col(region.begin());
                let row = row.saturating_add_signed(count);
                let line_char_indices = buffer.char_idx(row, col);
                let begin = line_char_indices.query;
                let last = begin
                    .saturating_add(region.len().saturating_sub(1))
                    .min(line_char_indices.last);

                Region::ii(begin, last)
            })
            .collect();
    }

    pub fn move_backward(&mut self) {
        self.translate_by(-1);
    }

    pub fn move_down(&mut self, buffer: &Buffer) {
        self.translate_by_line(buffer, 1);
    }

    pub fn move_forward(&mut self) {
        self.translate_by(1);
    }

    pub fn move_up(&mut self, buffer: &Buffer) {
        self.translate_by_line(buffer, -1);
    }

    pub fn scroll_down(&mut self, buffer: &Buffer, count: usize) {
        let max_y = buffer.len_lines().saturating_sub(2);

        self.position.y = self.position.y.saturating_add(count).min(max_y);
    }

    pub fn scroll_up(&mut self, count: usize) {
        self.position.y = self.position.y.saturating_sub(count);
    }

    pub fn scroll_left(&mut self, count: usize) {
        self.position.x = self.position.x.saturating_sub(count);
    }

    // TODO: need to find max value (requires getting length of each line that's being rendered)
    pub fn scroll_right(&mut self, count: usize) {
        self.position.x = self.position.x.saturating_add(count);
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
            let insert_idx = region.begin().saturating_add(diff);
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
