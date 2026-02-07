use crate::{region::Region, selection::Selection, selection_set::SelectionSet};
use camino::Utf8PathBuf;
use mkutils::{PointUsize, Scrollable, Utils};
use std::borrow::Cow;
use ulid::Ulid;

pub struct View {
    buffer_id: Ulid,
    filepath: Option<Utf8PathBuf>,
    scroll_offset: PointUsize,
    latest_buffer_draw_size: PointUsize,
    selection_set: SelectionSet,
}

impl View {
    const DEFAULT_NAME: &str = "untitled";
    const INITIAL_SCROLL_OFFSET: PointUsize = PointUsize::ZERO;
    const INITIAL_LATEST_BUFFER_DRAW_SIZE: PointUsize = PointUsize::ZERO;

    pub fn new(buffer_id: Ulid, filepath: Option<Utf8PathBuf>) -> Self {
        let scroll_offset = Self::INITIAL_SCROLL_OFFSET;
        let latest_buffer_draw_size = Self::INITIAL_LATEST_BUFFER_DRAW_SIZE;
        let selection_set = Self::initial_selection_set();

        Self {
            buffer_id,
            filepath,
            scroll_offset,
            latest_buffer_draw_size,
            selection_set,
        }
    }

    fn initial_selection_set() -> SelectionSet {
        Region::ZERO.convert::<Selection>().into()
    }

    pub const fn buffer_id(&self) -> Ulid {
        self.buffer_id
    }

    pub fn name(&self) -> Cow<'_, str> {
        let Some(filepath) = &self.filepath else { return Self::DEFAULT_NAME.to_cow_borrowed() };

        match filepath.unexpand_user() {
            Cow::Borrowed(filepath) => filepath.as_str().to_cow_borrowed(),
            Cow::Owned(filepath) => filepath.into_string().into_cow_owned(),
        }
    }

    pub const fn scroll_offset(&self) -> PointUsize {
        self.scroll_offset
    }

    pub fn set_latest_buffer_draw_size(&mut self, latest_buffer_draw_size: PointUsize) {
        self.latest_buffer_draw_size = latest_buffer_draw_size;
    }
}

impl Scrollable for View {
    fn scroll_offset_mut(&mut self) -> &mut PointUsize {
        &mut self.scroll_offset
    }

    fn latest_content_render_size(&self) -> PointUsize {
        self.latest_buffer_draw_size
    }
}
