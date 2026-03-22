use camino::Utf8Path;
use mkutils::{Rope, Utils};
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

    pub fn empty() -> Self {
        Self::new(Rope::new())
    }

    pub async fn from_filepath(filepath: &Utf8Path) -> Result<Self, IoError> {
        filepath
            .open_async()
            .await?
            .buf_reader_async()
            .to_rope_async()
            .await?
            .pipe_into(Self::new)
            .ok()
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
}
