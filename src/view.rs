use camino::Utf8PathBuf;
use mkutils::{PointUsize, Utils};
use std::borrow::Cow;
use ulid::Ulid;

pub struct View {
    buffer_id: Ulid,
    filepath: Option<Utf8PathBuf>,
    scroll_offset: PointUsize,
}

impl View {
    const DEFAULT_NAME: &str = "untitled";
    const INITIAL_SCROLL_OFFSET: PointUsize = PointUsize::ZERO;

    pub const fn new(buffer_id: Ulid, filepath: Option<Utf8PathBuf>) -> Self {
        let scroll_offset = Self::INITIAL_SCROLL_OFFSET;

        Self {
            buffer_id,
            filepath,
            scroll_offset,
        }
    }

    pub const fn buffer_id(&self) -> Ulid {
        self.buffer_id
    }

    pub fn name(&self) -> Cow<'_, str> {
        let Some(filepath) = &self.filepath else { return Self::DEFAULT_NAME.borrowed() };

        match filepath.unexpand_user() {
            Cow::Borrowed(filepath) => filepath.as_str().borrowed(),
            Cow::Owned(filepath) => filepath.into_string().owned(),
        }
    }

    pub const fn scroll_offset(&self) -> PointUsize {
        self.scroll_offset
    }
}
