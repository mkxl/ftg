use camino::Utf8Path;
use mkutils::{PointU16, PointUsize, Rope, Utils};
use ratatui::text::{Line as RatatuiLine, Span};
use std::io::Error as IoError;
use ulid::Ulid;

pub struct Buffer {
    buffer_id: Ulid,
    rope: Rope,
}

impl Buffer {
    fn new(rope: Rope) -> Self {
        let buffer_id = Ulid::new();

        Self { buffer_id, rope }
    }

    fn empty() -> Self {
        Self::new(Rope::new())
    }

    pub async fn from_filepath(filepath: &Utf8Path) -> Result<Self, IoError> {
        filepath
            .open_async()
            .await?
            .buf_reader_async()
            .rope_builder()
            .build()
            .await?
            .pipe_into(Self::new)
            .ok()
    }

    pub const fn buffer_id(&self) -> Ulid {
        self.buffer_id
    }

    pub fn size(&self) -> PointUsize {
        self.rope.size()
    }

    pub fn raw_ratatui_lines(&self, scroll_offset: PointUsize, size: PointU16) -> Vec<RatatuiLine<'_>> {
        let line_offsets = scroll_offset.y.range_from_len(size.y.into());
        let extended_grapheme_offsets = scroll_offset.x.range_from_len(size.x.into());
        let mut rope_lines = self.rope.lines(line_offsets, extended_grapheme_offsets);
        let mut raw_ratatui_lines = Vec::new();

        while let Some(rope_line) = rope_lines.next_line() {
            rope_line
                .map(|atom| Span::raw(atom.extended_grapheme()))
                .collect::<RatatuiLine>()
                .push_to(&mut raw_ratatui_lines);
        }

        raw_ratatui_lines
    }
}
