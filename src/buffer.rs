use camino::Utf8Path;
use mkutils::{PointU16, PointUsize, Utils};
use ratatui::text::{Line, Span};
use ropey::Rope;
use std::io::Error as IoError;
use ulid::Ulid;

pub struct Buffer {
    buffer_id: Ulid,
    rope: Rope,
    size: PointUsize,
}

impl Buffer {
    fn empty() -> Self {
        let buffer_id = Ulid::new();
        let rope = Rope::new();
        let size = PointUsize::ZERO;

        Self { buffer_id, rope, size }
    }

    fn new(rope: Rope) -> Self {
        let mut buffer = Self::empty();

        buffer.set_rope(rope);

        buffer
    }

    fn set_rope(&mut self, rope: Rope) {
        self.size = rope.as_rope_slice().num_lines_and_extended_graphemes();
        self.rope = rope;
    }

    pub const fn buffer_id(&self) -> Ulid {
        self.buffer_id
    }

    pub const fn size(&self) -> PointUsize {
        self.size
    }

    pub async fn from_filepath(filepath: &Utf8Path) -> Result<Self, IoError> {
        // NOTE-bb39cb: [.pin()] to avoid [https://rust-lang.github.io/rust-clippy/master/index.html#large_futures]
        filepath
            .rope::<{ Self::DEFAULT_ROPE_BUILDER_BUFFER_SIZE }>()
            .pin()
            .await?
            .pipe_into(Self::new)
            .ok()
    }

    pub fn lines(&self, scroll_offset: PointUsize, size: PointU16) -> Vec<Line<'_>> {
        let lines_index_range = scroll_offset.y.range_from_len(size.y.into());
        let extended_graphemes_index_range = scroll_offset.x.range_from_len(size.x.into());

        self.rope
            .as_rope_slice()
            .extended_graphemes_at_rect(lines_index_range, extended_graphemes_index_range)
            .map(|extended_graphemes| extended_graphemes.map(Span::from).collect())
            .collect()
    }
}
