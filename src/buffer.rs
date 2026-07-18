use anyhow::Error as AnyhowError;
use camino::Utf8Path;
use mkutils::{PointUsize, Rope, Utils};
use std::io::Error as IoError;
use ulid::Ulid;

pub struct Buffer {
    buffer_id: Ulid,
    rope: Rope,
}

impl Buffer {
    const CONTEXT_READ_FILEPATH_TEXT: &str = "read filepath text";

    fn from_rope(rope: Rope) -> Self {
        let buffer_id = Ulid::new();

        Self { buffer_id, rope }
    }

    pub fn new() -> Self {
        Self::from_rope(Rope::new())
    }

    async fn rope_from_filepath_impl(filepath: &Utf8Path) -> Result<Rope, IoError> {
        filepath.open_async().await?.buf_reader_async().to_rope_async().await
    }

    // TODO: use the mkutils context attribute macro here
    pub async fn rope_from_filepath(filepath: &Utf8Path) -> Result<Rope, AnyhowError> {
        Self::rope_from_filepath_impl(filepath)
            .await
            .context_path(Self::CONTEXT_READ_FILEPATH_TEXT, filepath)
    }

    pub const fn buffer_id(&self) -> Ulid {
        self.buffer_id
    }

    pub const fn rope(&self) -> &Rope {
        &self.rope
    }

    pub const fn rope_mut(&mut self) -> &mut Rope {
        &mut self.rope
    }

    pub fn size(&self) -> PointUsize {
        let mut size = self.rope.size();

        size.y.increment();

        size
    }

    pub fn set_rope(&mut self, rope: Rope) {
        self.rope = rope;
    }
}
